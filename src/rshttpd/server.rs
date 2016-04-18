extern crate regex;
extern crate time;
extern crate rustc_serialize;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::str;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::File;
use std::collections::HashMap;
use std::ffi::OsStr;
use self::regex::Regex;
use self::rustc_serialize::json::Json;

#[derive(Clone)]
pub struct Server {
	listen: String,
	docroot: String
}

impl Server {
	pub fn new() -> Server {
		let mut input = String::new();
		File::open("rshttpd.json").and_then(|mut f| {
			f.read_to_string(&mut input)
		}).unwrap();

		let data = Json::from_str(input.as_str()).unwrap();
		let config = data.as_object().expect("Config is not a valid JSON file.");

		let listen = config.get("listen").and_then(|json| {
			json.as_string()
		}).expect("'config.listen' was missing or is not a valid string");

		let docroot = config.get("docroot").and_then(|json| {
			json.as_string()
		}).expect("'config.docroot' was missing or is not a valid string");


		Server {
			listen: listen.to_string(),
			docroot: docroot.to_string()
		}
	}

	pub fn run(&self) {
		info!("Starting rshttpd. The time is {}", time::now_utc().ctime());
		info!("Binding to address {}", self.listen);

		let socket = TcpListener::bind(self.listen.as_str()).unwrap();

		for stream in socket.incoming() {
			match stream {
				Ok(stream) => {
					let clone = self.clone();
					thread::spawn(move || {
						clone.handle_client(stream)
					});
				}
				Err(err) => {
					error!("Socket error: {}", err);
				}
			}
		}

		drop(socket);
	}

	fn handle_client(&self, mut stream: TcpStream) {
		/*
		// TODO: ip address of client for access log
		match stream.peer_addr() {
			Ok(addr) => {
				println!("New client {}", addr);
			}
			Err(err) => {
				println!("Error peer_addr(): {}", err);
			}
		}
		*/

		let mut buffer: [u8; 1024] = [0; 1024];

		let size = match stream.read(&mut buffer) {
			Ok(s) => s,
			Err(_) => 0
		};

		if size > 0 {
			let s = match str::from_utf8(&buffer) {
				Ok(v) => v,
				Err(e) => {
					error!("Invalid UTF-8 sequence: {}", e);
					return ();
				}
			};

			let mut lines = s.lines();

			// we can unwrap, there has to be at least one line
			let first = lines.nth(0).unwrap();

			let re = Regex::new(r"^(\w+)\s+(\S+)\s+(\S+)$").unwrap();

			for cap in re.captures_iter(first) {
				let method = cap.at(1).unwrap_or("");
				let uri = cap.at(2).unwrap_or("");
				//let protocol = cap.at(3).unwrap_or("");

				if method == "GET" {
					let mut path = PathBuf::from(self.docroot.clone());

					if uri == "/" {
						path.push("index.html");
					}
					else {
						path.push(uri.trim_left_matches("/"));
					}

					let mut file = match File::open(path.as_path()) {
						Ok(f) => f,
						Err(err) => {
							error!("Error reading uri {}: {}", uri, err);
							self.respond(&mut stream, 404, HashMap::new(), self.get_message(404).as_bytes());
							return ();
						}
					};

					let mut content = Vec::<u8>::new();

					let filesize: usize = match file.read_to_end(&mut content) {
						Ok(n) => n,
						Err(err) => {
							error!("Error reading uri {}: {}", uri, err);
							self.respond(&mut stream, 500, HashMap::new(), self.get_message(500).as_bytes());
							return ();
						}
					};

					let size_str = format!("{}", filesize);

					let mut headers = HashMap::new();
					let extension = match path.extension() {
						Some(ext) => ext,
						None => OsStr::new("")
					};

					if extension.ne("") {
						headers.insert("Content-type", self.get_type(extension.to_str()));
					}

					headers.insert("Content-size", size_str.as_str());

					self.respond(&mut stream, 200, headers, content.as_slice());
				}
				else {
					self.respond(&mut stream, 501, HashMap::new(), self.get_message(501).as_bytes());
				}
			}
		}
		else {
			error!("Empty request");
			return ();
		}
	}

	fn respond(&self, stream: &mut TcpStream, status: u32, mut headers: HashMap<&str, &str>, content: &[u8] ) {
		let mut hstr = String::new();

		if !headers.contains_key("Content-type") {
			headers.insert("Content-type", "text/plain");
		}

		for (key, val) in headers {
			hstr.push_str(format!("{}: {}\r\n", key, val).as_str());
		}

		let resp = format!("HTTP/1.1 {} {}\r\n{}\r\n", status, self.get_message(status), hstr);

		// TODO: check if write was successful
		let _ = stream.write_all(&resp.into_bytes());
		let _ = stream.write_all(content);
	}

	fn get_type(&self, extension: Option<&str>) -> &str {
		return match extension {
			Some("gif") => "image/gif",
			Some("png") => "image/png",
			Some("jpg") => "image/jpeg",
			Some("txt") => "text/plain",
			Some("html") => "text/html",
			Some(_) => "text/plain",
			None => "text/plain"
		};
	}

	fn get_message(&self, code: u32) -> &str {
		return match code {
			100 => "Continue",
			101 => "Switching protocols",
			200 => "OK",
			201 => "Created",
			202 => "Accepted",
			203 => "Non-Authoritative Information",
			204 => "No Content",
			205 => "Reset Content",
			206 => "Partial Content",
			207 => "Partial Update OK",
			300 => "Multiple Choices",
			301 => "Moved Permanently",
			302 => "Moved Temporarily",
			303 => "See Other",
			304 => "Not Modified",
			305 => "Use Proxy",
			307 => "Temporary Redirect",
			400 => "Bad Request",
			401 => "Unauthorized",
			402 => "Payment Required",
			403 => "Forbidden",
			404 => "Not Found",
			405 => "Method Not Allowed",
			406 => "Not Acceptable",
			407 => "Proxy Authentication Required",
			408 => "Request Timeout",
			409 => "Conflict",
			410 => "Gone",
			411 => "Length Required",
			412 => "Precondition Failed",
			413 => "Request Entity Too Large",
			414 => "Request-URI Too Long",
			415 => "Unsupported Media Type",
			416 => "Requested range not satisfiable",
			417 => "Expectation Failed",
			418 => "Reauthentication Required",
			419 => "Proxy Reauthentication Required",
			500 => "Internal Server Error",
			501 => "Not Implemented",
			502 => "Bad Gateway",
			503 => "Service Unavailable",
			504 => "Gateway Timeout",
			505 => "HTTP Version Not Supported",
			506 => "Partial Update Not Implemented",
			_ => "UNKNOWN"
		}
	}
}
