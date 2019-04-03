
#[derive(Debug)]
pub struct GeneratorError( String );

impl From<std::fmt::Error> for GeneratorError
{
    fn from( _ : std::fmt::Error ) -> Self {
        GeneratorError( "Formatting error".to_string() )
    }
}

impl From<&str> for GeneratorError
{
    fn from( src : &str) -> Self {
        GeneratorError( src.to_string() )
    }
}

