extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use std::io::Read;

fn main() {
	println!("Starting rshttpd. The time is {}", time::now_utc().ctime());

	let socket = TcpListener::bind("127.0.0.1:8080").unwrap();

	println!("Listening on port 8080");

	for stream in socket.incoming() {
		match stream {
			Ok(stream) => {
				thread::spawn(move|| {
					handle_client(stream)
				});
			}
			Err(err) => {
				println!("Socket error: {}", err);
			}
		}
	}

	drop(socket);
}

fn handle_client(mut stream: TcpStream) {
	match stream.peer_addr() {
		Ok(addr) => {
			println!("New client {}", addr);
		}
		Err(err) => {
			println!("Error peer_addr(): {}", err);
		}
	}

	let mut buffer: [u8; 1024] = [0; 1024];

	let size = match stream.read(&mut buffer) {
		Ok(s) => s,
		Err(_) => 0
	};

	if size > 0 {
		let s = match str::from_utf8(&buffer) {
			Ok(v) => v,
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
		};

		println!("{}", s);
	}
}
