
use super::prelude::*;

pub trait DescriptorProtoExt
{
    fn get_nested_or_self<'a>(
        &'a self,
        name : &[&str]
    ) -> Option< TypeDescriptor<'a> >;
}

impl DescriptorProtoExt for protobuf::descriptor::DescriptorProto
{
    fn get_nested_or_self<'a>( &'a self, name : &[&str] ) -> Option< TypeDescriptor<'a> > {

        let ( first, remainder ) = name.split_first()
            .expect( "Name must not be empty" );

        if first != &self.get_name() {
            return None;
        }

        if remainder.len() == 0 {
            return Some( TypeDescriptor::Message( self ) );
        }

        self.get_nested_type()
                .iter()
                .filter_map( |nt| nt.get_nested_or_self( remainder ) )
                .nth(0)
    }
}
