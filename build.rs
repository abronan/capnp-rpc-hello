extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new().src_prefix("src").file("src/hello.capnp").run().unwrap();
}