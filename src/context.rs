
use crate::protos;
use crate::spec;
use crate::internal_utils::DescriptorProtoExt;

/// Holds context information for the whole plugin invocation.
#[derive(Clone, Copy)]
pub struct PluginContext<'a>
{
    /// Code generator request passed in by protoc.
    pub request : &'a protos::plugin::CodeGeneratorRequest,

    /// Parsed specification files.
    pub files : &'a Vec<spec::File<'a>>,
}

/// Holds context information for a single proto file.
#[derive(Clone, Copy)]
pub struct FileContext<'a>
{
    /// Proto file descriptor.
    pub descriptor : &'a protobuf::descriptor::FileDescriptorProto,

    /// Parent plugin context.
    pub plugin_context : PluginContext<'a>
}

/// Holds context information for a protobuf type.
#[derive(Clone)]
pub struct TypeContext<'a>
{
    /// Fully qualified absolute name of the type.
    pub full_name : String,

    /// Type descriptor.
    pub type_descriptor : TypeDescriptor<'a>,

    /// Type constructor specification.
    pub type_spec : Option< &'a spec::Type<'a> >,

    /// Parent file context.
    pub file_context : FileContext<'a>,
}

/// Type descriptor that may represent either a protobuf enum or message.
#[derive(Clone, Copy)]
pub enum TypeDescriptor<'a>
{
    Message( &'a protobuf::descriptor::DescriptorProto ),
    Enum( &'a protobuf::descriptor::EnumDescriptorProto ),
}

impl<'a> PluginContext<'a>
{
    /// Iterate all proto files that have been listed as generator output.
    pub fn iter_generated_files(
        &self
    ) -> impl IntoIterator<Item = FileContext<'a>>
    {
        self.request.get_file_to_generate()
            .iter()
            .map( |file| FileContext {
                descriptor: self.get_file_descriptor( file ).unwrap(),
                plugin_context: *self,
            } )
            .collect::<Vec<_>>()
    }

    /// Iterate all proto types that have been listed as generator output.
    pub fn iter_generated_types(
        &self
    ) -> impl IntoIterator<Item = ( TypeContext<'a>, &'a spec::Type<'a> )>
    {
        self.iter_generated_files()
            .into_iter()
            .flat_map( move |file_context| file_context.iter_generated_types() )
            .collect::<Vec<_>>()
    }

    /// Iterates through all constructors that should be generated.
    ///
    /// This includes all the constructors the type of which is included in the .proto files.
    pub fn iter_generated_constructors(
        &self
    ) -> impl IntoIterator<Item = ( TypeContext<'a>, &'a spec::Constructor<'a> )>
    {
        self.iter_generated_files()
            .into_iter()
            .flat_map( move |file_context| file_context.iter_generated_constructors() )
            .collect::<Vec<_>>()
    }

    /// Gets a type by its absolute name.
    pub fn get_type(
        &self,
        type_name : &str,
    ) -> Option< TypeContext<'a> >
    {
        if let Some( descriptors ) = self.get_type_descriptor( type_name ) {

            let ( file_context, descriptor ) = descriptors;
            let spec = self.get_type_spec( type_name );

            return Some( TypeContext {
                full_name : type_name.to_string(),
                type_spec: spec,
                type_descriptor : descriptor,
                file_context,
            } )
        }

        None
    }

    /// Gets a type by its relative name from an absolute source.
    ///
    /// Used to resolve type references in messages for example.
    /// These references may be relative to any of the parent packages.
    pub fn get_rel_type(
        &self,
        source_type : &str,
        type_name : &str,
    ) -> Option< TypeContext<'a> >
    {
        let mut source_type = source_type.split(".").collect::<Vec<_>>();

        loop {

            let candidate = format!(
                "{}.{}",
                source_type.join("."),
                type_name );

            if let Some( t ) = self.get_type( &candidate ) {
                return Some( t.clone() );
            }

            if let Some( _ ) = source_type.pop() {
                continue;
            }

            return None;
        }
    }

    /// Gets a type spec by its full name if one exists.
    fn get_type_spec(
        &self,
        type_name : &str
    ) -> Option<&'a spec::Type<'a>>
    {
        for file in self.files {

            // Ensure the file contains types belonging to this package.
            let package_start = format!( "{}.", file.package );
            if ! type_name.starts_with( &package_start ) {
                continue
            }

            // Resolve the relative name of the type if it exists within this file.
            let rel_name = &type_name[ package_start.len() .. ]
                .split(".")
                .collect::<Vec<_>>();

            // Currently nested types are not supported.
            for t in &file.types {
                if let Some( found ) = t.get_nested_or_self( rel_name ) {
                    return Some( found );
                }
            }
        }

        None
    }

    /// Gets a type descriptor by its full name if one exists.
    fn get_type_descriptor(
        &self,
        type_name : &str
    ) -> Option<( FileContext<'a>, TypeDescriptor<'a> )>
    {
        for file in self.request.get_proto_file() {

            // Ensure the file contains types belonging to this package.
            let package_start = format!( "{}.", file.get_package() );
            if ! type_name.starts_with( &package_start ) {
                continue
            }

            // Resolve the relative name of the type if it exists within this file.
            let rel_name = &type_name[ package_start.len() .. ]
                .split(".")
                .collect::<Vec<_>>();

            // Currently nested types are not supported.
            for t in file.get_message_type() {
                if let Some( found ) = t.get_nested_or_self( rel_name ) {
                    return Some( ( FileContext {
                        descriptor : file,
                        plugin_context: *self,
                    }, found ) );
                }
            }
        }

        None
    }

    /// Finds a file descriptor given a name.
    fn get_file_descriptor(
        &self,
        file_name : &str
    ) -> Option< &'a protobuf::descriptor::FileDescriptorProto >
    {
        self.request
            .get_proto_file()
            .into_iter()
            .filter( |f| f.get_name() == file_name )
            .nth(0)
            .clone()
    }
}

impl<'a> FileContext<'a>
{
    /// Iterates through all the types within this file for which constructors are defined.
    ///
    /// Each of these types is guaranteed to have a spec for it. The spec is returned through the
    /// iterator items so the caller doesn't need to unwrap the specs on the type context.
    pub fn iter_generated_types(
        &self
    ) -> impl IntoIterator<Item = ( TypeContext<'a>, &'a spec::Type<'a> )>
    {
        self.descriptor.get_message_type()
            .iter()
            .filter_map( |t| {
                let full_name = format!( "{}.{}",
                    self.descriptor.get_package(),
                    t.get_name() );
                match self.plugin_context.get_type_spec( &full_name ) {
                    None => None,
                    Some( type_spec ) => Some( ( TypeContext {
                        full_name: full_name,
                        type_descriptor: TypeDescriptor::Message( t ),
                        type_spec: Some( type_spec ),
                        file_context: *self,
                    }, type_spec ) )
                }
            } )
            .collect::<Vec<_>>()
    }

    /// Iterates through all constructors that should be generated based on the types within this
    /// file.
    ///
    /// This includes all the constructors the type of which is included in the .proto files.
    pub fn iter_generated_constructors(
        &self
    ) -> impl IntoIterator<Item = ( TypeContext<'a>, &'a spec::Constructor<'a> )>
    {
        self.iter_generated_types()
            .into_iter()
            .flat_map( move |(type_context, type_spec)|
                type_spec.constructors.iter().map( move |ctor| (type_context.clone(), ctor) ) )
            .collect::<Vec<_>>()
    }

}

impl<'a> TypeContext<'a>
{
    /// Gets the short name of the type.
    ///
    /// The type.full_name is an absolute name while the name returned by this
    /// function is just the individual name of the item, excluding the path.
    pub fn get_name( &self ) -> &str
    {
        match self.type_descriptor {
            TypeDescriptor::Message( m ) => m.get_name(),
            TypeDescriptor::Enum( e ) => e.get_name(),
        }
    }
}

