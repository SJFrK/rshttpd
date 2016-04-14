extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use std::io::Read;

fn main() {
	println!("Starting rhttpd. The time is {}", time::now_utc().ctime());
	
	let socket = TcpListener::bind("127.0.0.1:8080").unwrap();
	
    println!("Listening on port 8080");
    
    // accept connections and process them, spawning a new thread for each one
    for stream in socket.incoming() {
		match stream {
			Ok(stream) => {
				thread::spawn(move|| {
					// connection succeeded
					handle_client(stream)
				});
			}
			Err(err) => {
				/* connection failed */
				println!("Socket error: {}", err);
			}
		}
	}

	// close the socket server
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

/*
fn handle_client(mut stream: io::TcpStream) -> io::IoResult<()> {
    let mut buf: Buf = [0u8, ..10240];
    let (child_tx, parent_rx) = channel::<Buf>();
    let (parent_tx, child_rx) = channel::<Buf>();

    spawn(proc() {
        std::task::deschedule();
        for mut buf in child_rx.iter() {
            for _ in range::<u8>(0, 20) {
                buf.reverse();
            }
            child_tx.send(buf);
        };
    });
    
    loop {
        let got = try!(stream.read(buf));
        if got == 0 {
            // Is it possible? Or IoError will be raised anyway?
            break
        }
        // outsource CPU-heavy work to separate task, because current green+libuv
        // implementation bind all IO tasks to one scheduler (rust 0.11)
        // see https://botbot.me/mozilla/rust/2014-08-01/?msg=18995736&page=11
        parent_tx.send(buf);
        let to_send: Buf = parent_rx.recv();
        try!(stream.write(to_send.slice(0, got)));
    }
    Ok(())
}

    */
