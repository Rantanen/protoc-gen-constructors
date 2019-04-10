
use std::fmt::Write;
use crate::prelude::*;
use inflector::cases::camelcase::to_camel_case;

pub fn write_javadoc(
    out : &mut Write,
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

