#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;

mod rshttpd;

fn main() {
	// TODO: switch to a proper file logger
	env::set_var("RUST_LOG", "info,warn,error,debug,trace");

	match env_logger::init() {
		Ok(_) => {},
		Err(e) => panic!("Could not initialize logger: {}", e)
	}

	let server = rshttpd::server::Server::new();
	server.run();
}
