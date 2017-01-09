use std::ascii::AsciiExt;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::string::String;
use std::vec::Vec;

pub enum Protocol {
	Http10,
	Http11,
}

pub enum Method {
	Get,
	Post,
}

pub enum Connection {
	Close,
	KeepAlive,
}

pub struct Request {
	pub remote_addr: Option<SocketAddr>,
	pub protocol: Option<Protocol>,
	pub method: Option<Method>,
	pub connection: Connection,
	pub header: Vec<Vec<u8>>,
}

impl Request {
	fn new(stream: &TcpStream) -> Request {
		Request {
			remote_addr: stream.peer_addr().ok(),
			protocol: None,
			method: None,
			connection: Connection::Close,
			header: Vec::new(),
		}
	}
	pub fn get_url(&self) -> Option<&[u8]> {
		if ! self.header.is_empty() {
			let first_line = &self.header[0];
			if let Some(pos1) = first_line[..].iter().position(|&x| x == 32) {
				if let Some(pos2) = first_line[..].iter().rposition(|&x| x == 32) {
					if pos1 + 1 < pos2 {
						return Some(&first_line[pos1 + 1..pos2])
					}
				}
			}
		}
		None
	}
	pub fn create_response(&self) -> Response {
		Response::new()
	}
}

pub struct Response {
	pub content: Option<Vec<u8>>,
}

impl Response {
	fn new() -> Response {
		Response {
			content: None,
		}
	}
}

pub trait Handler {
	fn handle(&self, &Request) -> Response;
}

pub struct HttpHandler<T> {
	handler: T,
	offset: usize,
	buffer: [u8; 8192],
}

impl<T: Handler> HttpHandler<T> {

	pub fn new(h: T) -> HttpHandler<T> {
		HttpHandler {
			handler: h,
			offset: 0,
			buffer: [0; 8192],
		}
	}

	pub fn read_line(&mut self, stream: &mut TcpStream) -> Option<Vec<u8>> {
		loop {
			if self.offset > 0 {
				if let Some(pos) = self.buffer[..self.offset].into_iter().position(|&x| x == 10) {
					let eol = if pos > 0 && self.buffer[pos - 1] == 13 { pos - 1 } else { pos };
					let line = self.buffer[0..eol].to_vec();
					if pos + 1 < self.offset {
						for i in pos + 1..self.offset {
							self.buffer[i - pos - 1] = self.buffer[i];
						}
						self.offset = self.offset - pos - 1;
					} else {
						self.offset = 0;
					}
					return Some(line)
				}
			}
			if self.offset < self.buffer.len() {
				let size = stream.read(&mut self.buffer[self.offset..]).unwrap();
				if size == 0 { break; }
				self.offset = self.offset + size;
			} else {
				break;
			}
		}
		None
	}

	pub fn handle(&mut self, mut stream: TcpStream) {

		let mut request = Request::new(&stream);

		while let Some(line) = self.read_line(&mut stream) {
			if line.len() == 0 { break; }
			if request.header.is_empty() {
				if let Some(pos) = line.iter().position(|&x| x == 32) {
					if line[..pos].eq_ignore_ascii_case(b"GET") {
						request.method = Some(Method::Get);
					} else if line[..pos].eq_ignore_ascii_case(b"POST") {
						request.method = Some(Method::Post);
					}
				}
				if let Some(pos) = line.iter().rposition(|&x| x == 32) {
					if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.0") {
						request.protocol = Some(Protocol::Http10);
					} else if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.1") {
						request.protocol = Some(Protocol::Http11);
					}
				}
			}
			request.header.push(line);
		}

		//if self.offset > 0 {
		//	for i in self.buffer[..self.offset].iter() {
		//		println!("{}", i);
		//	}
		//}

		//let _ = stream.write(b"HTTP/1.1 400 Bad Request\r\n\r\n");

		let response = self.handler.handle(&request);
		if let Some(content) = response.content {
			let mut header = String::new();
			header.push_str("HTTP/1.1 200 OK\r\n");
			header.push_str("Server: Rust 1.13.0\r\n");
			header.push_str("Content-Type: text/html; charset=UTF-8\r\n");
			header.push_str("Content-Length: ");
			header.push_str(content.len().to_string().as_str());
			header.push_str("\r\n");
			//header.push_str("Connection: keep-alive\r\n");
			header.push_str("Connection: close\r\n");
			header.push_str("\r\n");
			let _ = stream.write(header.as_bytes());
			let _ = stream.write(content.as_slice());
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
	}
}
