
use std::path::Path;
use std::fmt::Write;

use protoc_gen_constructors::prelude::*;
use protoc_gen_constructors::documentation::write_javadoc;

use inflector::cases::camelcase::to_camel_case;
use inflector::cases::pascalcase::to_pascal_case;

// Process the input through the library.
fn main() { process( run ) }

/// Runs the code generation.
fn run(
    context : &PluginContext
) -> Result<CodeGeneratorResponse, GeneratorError>
{
    // We'll need to return a CodeGeneratorResponse to protoc as the plugin response.
    let mut response = CodeGeneratorResponse::new();

    // Java generator uses insertion points to inject the functions into the types. The responses
    // we give back to protoc just define code for these insertion points for each type.

    for ( type_context, ctor ) in context.iter_generated_constructors() {

        // We need to tell protoc which file the insertion point is in.  For this reason we'll
        // need to mirror the output path of the original java generator.
        //
        // This output path uses the java package (net.foo.bar) as path (net/foo/bar) and the
        // final file name is based on the outer class name.
        let file_descriptor = &type_context.file_context.descriptor;
        let output_path = format!( "{}/{}.java",
                get_java_package( &file_descriptor ).replace(".", "/"),
                get_outer_class( &file_descriptor ) );

        // Generate the constructor code.
        //
        // We'll generate a constructor for the builder (which has the setters) and a
        // constructor for the class (which just delegates to the builder).
        let builder_code = get_builder_ctor( &type_context, ctor )?;
        let class_code = get_class_ctor( &type_context, ctor )?;

        // Add both of the constructors as responses to the total response object.

        let mut ctor_response = CodeGeneratorResponse_File::new();
        ctor_response.set_name( output_path.clone() );
        ctor_response.set_insertion_point(
                format!("builder_scope:{}", type_context.full_name ) );
        ctor_response.set_content( builder_code );
        response.mut_file().push( ctor_response );

        let mut ctor_response = CodeGeneratorResponse_File::new();
        ctor_response.set_name( output_path );
        ctor_response.set_insertion_point(
                format!("class_scope:{}", type_context.full_name ) );
        ctor_response.set_content( class_code );
        response.mut_file().push( ctor_response );
    }

    Ok( response )
}

/// Creates the constructor defined on the `Builder` class.
///
/// This is the constructor that does the actual field setting.
fn get_builder_ctor( 
    type_context : &TypeContext,
    ctor : &spec::Constructor
) -> Result<String, GeneratorError>
{
    // Format the constructor name and parameter list.
    //
    // Both the constructor and parameter names are in camel case in Java.
    let ctor_name = to_camel_case( ctor.name );
    let param_list = utils::join(
        &ctor.params, ", ", |p| format!( "{} {}",
           get_type( &p.param_type ),
           to_camel_case( &p.name ) ) );

    let mut out = String::new();
    write_javadoc( &mut out, ctor )?;
    writeln!( out, "public static Builder {}({}) {{", ctor_name, param_list )?;
    writeln!( out, "  Builder _builder = new Builder();" )?;

    for initializer in &ctor.initializers
    {
        // For the setters we'll use pascal casing because the initial `set` takes care of the
        // lower case portion of the camel casing.
        writeln!( out, "  _builder.set{}({});",
            to_pascal_case( initializer.field ),
            get_value( type_context, &initializer.value ),
        )?;
    }

    writeln!( out, "  return _builder;" )?;
    writeln!( out, "}}" )?;

    Ok(out)
}

/// Creates the constructor defined on the message class.
///
/// The constructor on the message just delegates to the Builder constructor.
fn get_class_ctor( 
    type_context : &TypeContext,
    ctor : &spec::Constructor
) -> Result<String, GeneratorError>
{
    // Class names in Java are in pascal case, everything else here is camel case.
    let class_name = to_pascal_case( type_context.get_name() );
    let ctor_name = to_camel_case( ctor.name );
    let param_list = utils::join( &ctor.params, ", ", |p| format!( "{} {}",
                   get_type( &p.param_type ),
                   to_camel_case( &p.name ) ) );
    let arg_list = utils::join( &ctor.params, ", ", |p| format!( "{}",
                   to_camel_case( &p.name ) ) );

    // The class constructor is a simple delegation to the builder constructor and then invoking
    // `.build()` on the received builder. The builder is an inner class so it's even named the
    // same for all classes.
    let mut out = String::new();
    write_javadoc( &mut out, ctor )?;
    writeln!( out, "public static {} {}({}) {{", class_name, ctor_name, param_list )?;
    writeln!( out, "  return Builder.{}({}).build();", ctor_name, arg_list )?;
    writeln!( out, "}}" )?;

    Ok(out)
}

/// Gets the Java package.
fn get_java_package(
    descriptor : &protobuf::descriptor::FileDescriptorProto
) -> &str
{
    // Prefer the explicit java package specified as an option in the .proto file.
    let opts = descriptor.get_options();
    if opts.has_java_package() {
        return opts.get_java_package()
    }

    // In case no java package is defined in the proto file, use the normal package name.
    if descriptor.has_package() {
        return descriptor.get_package();
    }

    // No package defined.
    ""
}

/// Gets the Java outer class name.
fn get_outer_class(
    descriptor : &protobuf::descriptor::FileDescriptorProto
) -> &str
{
    // Prefer the explicit outer class name defined as an option in the .proto file
    let opts = descriptor.get_options();
    if opts.has_java_outer_classname() {
        return opts.get_java_outer_classname()
    }

    // No outer class name defined. Use the file name.
    let file = Path::new( descriptor.get_name() );
    return file.file_stem().and_then( |stem| stem.to_str() ).unwrap()
}

/// Convert Protobuf types into Java types.
fn get_type( 
    param_type : &spec::ParamType
) -> String
{
    match param_type {
        spec::ParamType::Int32 => "int".to_string(),
        spec::ParamType::String => "String".to_string(),
        spec::ParamType::Custom( name ) => name.to_string(),
    }
}

/// Convert constructor value expression into Java expression.
fn get_value( context : &TypeContext, expr : &spec::Expr ) -> String
{
    match expr {

        // Basic expressions.
        spec::Expr::Bool( b ) => format!( "{:?}", b ),
        spec::Expr::Integer( i ) => format!( "{}", i ),
        spec::Expr::Float( f ) => format!( "{}", f ),
        spec::Expr::Ref( r ) => to_camel_case( r ),
        spec::Expr::Enum( e ) => format!( "{}.{}", e.enum_name, e.value_name ),

        // Function calls are a bit more complex.
        spec::Expr::Call( c ) => {

            // The function may be scoped (Type::Function) or stand-alone (Function).
            //
            // For now the stand-alone functions are unimplemented. These are meant to be utility
            // functions that would be defined by hand for each language. Alternatively we'll never
            // use these and just define some `INTERNAL` type name, etc. that implements these
            // functions.
            let func = match c.type_name {
                Some( t ) => format!( "{}.{}", t, to_camel_case(c.func_name) ),
                None => unimplemented!(),
            };

            // Each function parameter is resolved with a recursive call to this
            // function.
            let params = c.args
                .iter()
                .map( |v| get_value( context, v ) )
                .collect::<Vec<_>>()
                .join(", ");

            format!( "{}({})", func, params )
        }
    }
}
