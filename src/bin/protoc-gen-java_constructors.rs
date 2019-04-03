
use std::path::Path;
use std::fmt::Write;

use protoc_gen_constructors::prelude::*;

use inflector::cases::camelcase::to_camel_case;
use inflector::cases::pascalcase::to_pascal_case;

fn main() { process( run ) }

fn run(
    context : &PluginContext
) -> Result<CodeGeneratorResponse, GeneratorError>
{
    let mut response = CodeGeneratorResponse::default();
    for file_context in context.iter_generated_files() {
        for ( type_context, spec ) in file_context.iter_generated_types() {
            for ctor in &spec.constructors {

                let package_path = get_package( &file_context.descriptor )
                        .replace( ".", "/" );
                let output_path = format!( "{}/{}.java",
                        package_path,
                        get_outer_class( &file_context.descriptor ) );

                let builder_code = get_builder_ctor( &type_context, ctor )?;

                let mut ctor_response = CodeGeneratorResponse_File::new();
                ctor_response.set_name( output_path.clone() );
                ctor_response.set_insertion_point( format!("builder_scope:{}", type_context.full_name ) );
                ctor_response.set_content( builder_code );
                response.mut_file().push( ctor_response );

                let class_code = get_class_ctor( &type_context, ctor )?;

                let mut ctor_response = CodeGeneratorResponse_File::new();
                ctor_response.set_name( output_path );
                ctor_response.set_insertion_point( format!("class_scope:{}", type_context.full_name ) );
                ctor_response.set_content( class_code );
                response.mut_file().push( ctor_response );
            }
        }
    }

    Ok( response )
}

fn get_builder_ctor( 
    type_context : &TypeContext,
    ctor : &spec::Constructor
) -> Result<String, GeneratorError>
{
    let mut result = String::new();
    let mut out = IndentingWriter::new( &mut result, "  " );

    write_javadoc( &mut out, ctor )?;

    let ctor_name = to_pascal_case( ctor.name );
    let param_list = utils::join( &ctor.params, ", ", |p| format!( "{} {}",
                   get_type( &p.param_type ),
                   to_camel_case( &p.name ) ) );
    writeln!( out, "public static Builder {}({}) {{", ctor_name, param_list )?;
    out.indent();
    writeln!( out, "Builder _builder = new Builder();" )?;

    for initializer in &ctor.initializers {
        writeln!( out, "_self.set{}({});",
            to_pascal_case( initializer.field ),
            get_value( type_context, &initializer.value ),
        )?;
    }

    writeln!( out, "return _builder;" )?;

    out.unindent();
    writeln!( out, "}}" )?;

    Ok(result)
}

fn get_class_ctor( 
    type_context : &TypeContext,
    ctor : &spec::Constructor
) -> Result<String, GeneratorError>
{
    let mut result = String::new();
    let mut out = IndentingWriter::new( &mut result, "  " );

    write_javadoc( &mut out, ctor )?;

    let class_name = to_pascal_case( type_context.get_name() );
    let ctor_name = to_pascal_case( ctor.name );
    let param_list = utils::join( &ctor.params, ", ", |p| format!( "{} {}",
                   get_type( &p.param_type ),
                   to_camel_case( &p.name ) ) );
    let arg_list = utils::join( &ctor.params, ", ", |p| format!( "{}",
                   to_camel_case( &p.name ) ) );

    writeln!( out, "public static {} {}({}) {{", class_name, ctor_name, param_list )?;
    writeln!( out, "  return Builder.{}({}).build();", ctor_name, arg_list )?;
    writeln!( out, "}}" )?;

    Ok(result)
}

fn write_javadoc(
    out : &mut IndentingWriter,
    ctor : &spec::Constructor
) -> Result<(), GeneratorError>
{
    let has_ctor_doc = ctor.documentation.is_some();
    let has_param_doc = ctor.params.iter().any( |p| p.documentation.is_some() );
    if ! has_ctor_doc && ! has_param_doc {
        return Ok(());
    }

    writeln!( out, "/**" )?;
    if let Some( doc ) = &ctor.documentation {
        for line in &doc.lines {
            writeln!( out, " * {}", line)?;
        }
    }

    if has_ctor_doc && has_param_doc {
        writeln!( out, " *" )?;
    }

    for param in &ctor.params {
        if let Some( doc ) = &param.documentation {
            writeln!( out, " * @param {}", to_camel_case( param.name ) )?;
            for line in &doc.lines {
                writeln!( out, " *        {}", line)?;
            }
        }
    }
    writeln!( out, " */" )?;

    Ok(())
}

fn get_package(
    descriptor : &protobuf::descriptor::FileDescriptorProto
) -> &str
{
    // Prefer the defined java package.
    let opts = descriptor.get_options();
    if opts.has_java_package() {
        return opts.get_java_package()
    }

    // In case no java package is defined in the proto file,
    // use the normal package name.
    if descriptor.has_package() {
        return descriptor.get_package();
    }

    // No package defined.
    ""
}

fn get_outer_class(
    descriptor : &protobuf::descriptor::FileDescriptorProto
) -> &str
{
    // Prefer the defined java outer class name.
    let opts = descriptor.get_options();
    if opts.has_java_outer_classname() {
        return opts.get_java_outer_classname()
    }

    // No outer class name defined. Use the file name.
    let file = Path::new( descriptor.get_name() );
    return file.file_stem().and_then( |stem| stem.to_str() ).unwrap()
}

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

fn get_value( context : &TypeContext, expr : &spec::Expr ) -> String
{
    match expr {
        spec::Expr::Bool( b ) => format!( "{:?}", b ),
        spec::Expr::Integer( i ) => format!( "{}", i ),
        spec::Expr::Float( f ) => format!( "{}", f ),
        spec::Expr::Ref( r ) => r.to_string(),
        spec::Expr::Enum( e ) => format!( "{}.{}", e.enum_name, e.value_name ),
        spec::Expr::Call( c ) => {

            let func = match c.type_name {
                Some( t ) => format!( "{}.{}", t, c.func_name ),
                None => c.func_name.to_string(),
            };

            let params = c.args
                .iter()
                .map( |v| get_value( context, v ) )
                .collect::<Vec<_>>()
                .join(", ");

            format!( "{}({})", func, params )
        }
    }
}
