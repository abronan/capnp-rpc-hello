use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use hello_capnp::hello;

use gj::{EventLoop, Promise};

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} client HOST:PORT NAME", args[0]);
        return;
    }

    EventLoop::top_level(move |wait_scope| -> Result<(), ::capnp::Error> {
            use std::net::ToSocketAddrs;
            let mut event_port = try!(::gjio::EventPort::new());
            let network = event_port.get_network();

            let addr = try!(args[2].to_socket_addrs())
                .next()
                .expect("could not parse address");

            let address = network.get_tcp_address(addr);
            let stream = try!(address.connect().wait(wait_scope, &mut event_port));

            let network = Box::new(twoparty::VatNetwork::new(stream.clone(),
                                                             stream,
                                                             rpc_twoparty_capnp::Side::Client,
                                                             Default::default()));

            let mut rpc_system = RpcSystem::new(network, None);
            let hello: hello::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            let mut request = hello.hello_request();
            request.get().set_name(&args[3]);

            let _result = request.send()
                .promise
                .then(|response| {
                    let response = pry!(pry!(response.get()).get_response());
                    println!("{}", response);
                    Promise::ok(())
                })
                .wait(wait_scope, &mut event_port);

            Ok(())
        })
        .expect("top level error");
}
