use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use hello_capnp::hello;

use gj::{EventLoop, Promise, TaskReaper, TaskSet};

struct HelloImpl;

impl hello::Server for HelloImpl {
    fn hello(&mut self,
             params: hello::HelloParams,
             mut results: hello::HelloResults)
             -> Promise<(), ::capnp::Error> {

        println!("received a request for greetings!");

        let greeting: &str = "Hello ";
        let name: &str = pry!(pry!(params.get()).get_name());
        let response = format!("{}{}!", greeting, name);

        results.get().set_response(&response);

        Promise::ok(())
    }
}

pub fn accept_loop(listener: ::gjio::SocketListener,
                   mut task_set: TaskSet<(), ::capnp::Error>,
                   helloc: hello::Client)
                   -> Promise<(), ::std::io::Error> {
    listener.accept().then(move |stream| {
        let mut network = twoparty::VatNetwork::new(stream.clone(),
                                                    stream,
                                                    rpc_twoparty_capnp::Side::Server,
                                                    Default::default());

        let disconnect_promise = network.on_disconnect();

        let rpc_system = RpcSystem::new(Box::new(network), Some(helloc.clone().client));

        task_set.add(disconnect_promise.attach(rpc_system));
        accept_loop(listener, task_set, helloc)
    })
}

struct Reaper;

impl TaskReaper<(), ::capnp::Error> for Reaper {
    fn task_failed(&mut self, error: ::capnp::Error) {
        println!("Task failed: {}", error);
    }
}

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server ADDRESS[:PORT]", args[0]);
        return;
    }

    EventLoop::top_level(move |wait_scope| -> Result<(), ::capnp::Error> {
            use std::net::ToSocketAddrs;
            let mut event_port = try!(::gjio::EventPort::new());
            let network = event_port.get_network();

            let addr = try!(args[2].to_socket_addrs())
                .next()
                .expect("could not parse address");

            let mut address = network.get_tcp_address(addr);
            let listener = try!(address.listen());

            let hello_server = hello::ToClient::new(HelloImpl).from_server::<::capnp_rpc::Server>();

            let task_set = TaskSet::new(Box::new(Reaper));

            try!(accept_loop(listener, task_set, hello_server).wait(wait_scope, &mut event_port));

            Ok(())
        })
        .expect("top level error");
}
