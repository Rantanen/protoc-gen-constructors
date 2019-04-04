
use std::fmt::Write;

use protoc_gen_constructors::prelude::*;
use protoc_gen_constructors::documentation::write_javadoc;

use inflector::cases::camelcase::to_camel_case;
use inflector::cases::pascalcase::to_pascal_case;

// Delegate to the library.
fn main() { process( run ) }

/// Runs the generation.
fn run(
    context : &PluginContext
) -> Result<CodeGeneratorResponse, GeneratorError>
{
    // protoc expects one response so gather the code from all generated files
    // under the same one.
    let mut response = CodeGeneratorResponse::default();
    for file_context in context.iter_generated_files() {

        let mut result = String::new();
        let mut out = IndentingWriter::new( &mut result, "    " );

        // We'll need to require the original protobuf generated file so we can augment the types
        // there.
        //
        // The file we are generating is placed in the same folder so we'll just need the file name
        // here, not the full path.
        let stem = std::path::Path::new( file_context.descriptor.get_name() )
            .file_stem()
            .and_then( |stem| stem.to_str() )
            .unwrap();
        let package = format!( "{}", stem.to_lowercase() );
        writeln!( out, "var __ = require('./{}_pb');", package )?;
        writeln!( out, "" )?;

        for ( type_context, spec ) in file_context.iter_generated_types() {
            for ctor in &spec.constructors {
                write_ctor( &mut out, &type_context, ctor )?;
            }
        }

        // Individual file responses are added to the total response we'll
        // return to protoc in the end.
        let mut ctor_response = CodeGeneratorResponse_File::new();
        let output_path = format!( "{}_pb-constructors.js",
            package );
        ctor_response.set_name(output_path);
        ctor_response.set_content(result);
        response.mut_file().push(ctor_response);
    }

    Ok( response )
}

/// Writes the constructor implementation.
fn write_ctor(
    out : &mut IndentingWriter,
    type_context : &TypeContext,
    ctor : &spec::Constructor
) -> Result<(), GeneratorError>
{
    // Since JS is a dynamic language, we'll just slap the constructors
    // on each type by hand.

    write_javadoc( out, ctor )?;

    let class_name = to_pascal_case( type_context.get_name() );
    let ctor_name = to_camel_case( ctor.name );
    let param_list = utils::join( &ctor.params, ", ", |p| format!( "{}",
                   to_camel_case( &p.name ) ) );

    // The constructor should look like:
    //
    // ```
    // __.Foo.CtorName = function CtorName(a, b) {
    //     var _self = new __.Foo();
    //     _self.setFieldA(a);
    //     _self.setFieldB(b);
    //     return _self;
    // }
    // ```

    writeln!( out, "__.{}.{} = function {}({}) {{",
        class_name,
        ctor_name,
        ctor_name,
        param_list )?;
    out.indent();

    writeln!( out, "var _self = new __.{}();", class_name )?;

    for initializer in &ctor.initializers {
        writeln!( out, "_self.set{}({});",
            to_pascal_case( initializer.field ),
            get_value( type_context, &initializer.value ),
        )?;
    }

    writeln!( out, "return _self;" )?;

    out.unindent();
    writeln!( out, "}}" )?;

    Ok(())
}

/// Turns a value expression into JS value.
fn get_value( context : &TypeContext, expr : &spec::Expr ) -> String
{
    match expr {
        spec::Expr::Bool( b ) => format!( "{:?}", b ),
        spec::Expr::Integer( i ) => format!( "{}", i ),
        spec::Expr::Float( f ) => format!( "{}", f ),
        spec::Expr::Ref( r ) => to_camel_case( r ),
        spec::Expr::Enum( e ) => format!( "{}.{}", e.enum_name, e.value_name ),
        spec::Expr::Call( c ) => {

            let func = match c.type_name {
                Some( t ) => format!( "__.{}.{}", t, to_camel_case( c.func_name ) ),
                None => to_camel_case( c.func_name ),
            };

            // The call parameters are resolved recursively.
            let params = c.args
                .iter()
                .map( |v| get_value( context, v ) )
                .collect::<Vec<_>>()
                .join(", ");

            format!( "{}({})", func, params )
        }
    }
}
