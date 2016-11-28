extern crate capnp;
extern crate capnp_rpc;

#[macro_use]
extern crate gj;
extern crate gjio;

pub mod hello_capnp {
    include!(concat!(env!("OUT_DIR"), "/hello_capnp.rs"));
}

pub mod server;
pub mod client;

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() >= 2 {
        match &args[1][..] {
            "client" => return client::main(),
            "server" => return server::main(),
            _ => (),
        }
    }
}
