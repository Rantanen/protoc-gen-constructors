use std::fmt::Write;

mod internal_utils;

pub mod protos;
pub mod spec;
pub mod error;
pub mod utils;
pub mod context;
pub mod documentation;

pub mod prelude {

    pub use super::error::*;
    pub use super::context::*;

    pub use super::protos::plugin::{CodeGeneratorResponse, CodeGeneratorResponse_File};

    pub use super::IndentingWriter;
    pub use super::process;
    pub use super::utils;

    pub use super::spec;
}

pub struct IndentingWriter<'a> {
    target : &'a mut Write,
    indent : &'a str,
    indent_count : usize,
    requires_indent : bool,
}

impl<'a> IndentingWriter<'a> {

    pub fn new( target: &'a mut Write, indent: &'a str ) -> IndentingWriter<'a> {
        IndentingWriter { target, indent, indent_count: 0, requires_indent: true }
    }

    pub fn indent( &mut self ) {
        self.indent_count += 1;
    }

    pub fn unindent( &mut self ) {
        self.indent_count -= 1;
    }
}

impl<'a> std::fmt::Write for IndentingWriter<'a> {

    fn write_str( &mut self, s : &str ) -> Result<(), std::fmt::Error>
    {
        let mut s = s;

        if s.is_empty() {
            return Ok(());
        }

        if self.requires_indent {
            self.target.write_str( &self.indent.repeat( self.indent_count ) )?;
            self.requires_indent = false;
        }

        while let Some( idx ) = s.find( "\n" ) {

            self.target.write_str( &s[..idx+1] )?;

            s = &s[1+idx..];
            if s.is_empty() {
                self.requires_indent = true;
                return Ok(());
            } else {
                self.target.write_str( &self.indent.repeat( self.indent_count ) )?;
            }
        }

        self.target.write_str( s )
    }
}


pub fn process<F>(f: F)
    where F:Fn(
        &context::PluginContext
    ) -> Result<protos::plugin::CodeGeneratorResponse, error::GeneratorError>
{

    // Deserialize the code generator request protoc is giving us.
    let request : protos::plugin::CodeGeneratorRequest =
        protobuf::parse_from_reader( &mut std::io::stdin() )
            .expect( "Bad request" );

    // Parse the constructors.
    let spec_files = request.get_parameter()
        .split(",")
        .map( |file| std::fs::read_to_string(file).expect( "Could not read" ) )
        .collect::<Vec<_>>();
    let files = spec_files.iter()
        .map(|f| spec::file(f).expect("Cold not parse"))
        .collect::<Vec<_>>();

    let context = context::PluginContext {
        request: &request,
        files: &files,
    };

    let result = f( &context );

    use protobuf::Message;
    result.unwrap()
        .write_to_writer( &mut std::io::stdout() )
        .expect( "Failed to write to stdout" );
}

