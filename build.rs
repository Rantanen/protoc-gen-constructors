
extern crate protobuf_codegen_pure;
extern crate peg;

fn main()
{
    protobuf_codegen_pure::run( protobuf_codegen_pure::Args {

        out_dir : "src/protos",
        input: &[
            "protos/google/protobuf/compiler/plugin.proto",
            "protos/google/protobuf/descriptor.proto" ],
        includes: &[ "protos" ],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
    } ).expect( "protoc" );

    peg::cargo_build( "src/spec.rustpeg" );
}
