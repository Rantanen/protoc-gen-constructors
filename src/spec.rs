
#[derive(Debug)]
pub struct File<'a> {
    pub package : &'a str,
    pub types : Vec<Type<'a>>
}

#[derive(Debug)]
pub struct Type<'a> {
    pub name : String,
    pub constructors: Vec<Constructor<'a>>,
    pub nested_types: Vec<Type<'a>>,
}

#[derive(Debug)]
pub struct Constructor<'a> {
    pub documentation : Option<Documentation<'a>>,
    pub name : &'a str,
    pub params : Vec<Parameter<'a>>,
    pub initializers : Vec<Initializer<'a>>,
}

#[derive(Debug)]
pub struct Documentation<'a> {
    pub lines : Vec<&'a str>,
}

#[derive(Debug)]
pub struct Parameter<'a> {
    pub documentation : Option<Documentation<'a>>,
    pub name : &'a str,
    pub param_type : ParamType<'a>,
}

#[derive(Debug)]
pub enum ParamType<'a> {
    Int32,
    String,
    Custom( &'a str ),
}

#[derive(Debug)]
pub struct Initializer<'a> {
    pub field : &'a str,
    pub value : Expr<'a>,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Call( Call<'a> ),
    Enum( EnumValue<'a> ),
    Bool( bool ),
    Integer( i64 ),
    Float( f64 ),
    Ref( &'a str ),
}

#[derive(Debug)]
pub struct Call<'a> {
    pub type_name : Option<&'a str>,
    pub func_name : &'a str,
    pub args : Vec<Expr<'a>>
}

#[derive(Debug)]
pub struct EnumValue<'a> {
    pub enum_name : &'a str,
    pub value_name : &'a str,
}

impl<'a> Type<'a>
{
    pub fn get_nested_or_self( &self, name : &[&str] ) -> Option< &Type<'a> > {

        let ( first, remainder ) = name.split_first()
            .expect( "Name must not be empty" );

        if first != &self.name {
            return None;
        }

        if remainder.len() == 0 {
            return Some( self );
        }

        self.nested_types
                .iter()
                .filter_map( |nt| nt.get_nested_or_self( remainder ) )
                .nth(0)
    }
}

include!(concat!(env!("OUT_DIR"), "/spec.rs"));
