use std::ascii::AsciiExt;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::string::String;
use std::vec::Vec;

#[derive(Copy,Clone)]
pub enum Protocol {
	Http10,
	Http11,
}

#[derive(Copy,Clone)]
pub enum Method {
	GET,
	POST,
}

#[derive(Copy,Clone)]
pub enum Connection {
	Close,
	KeepAlive,
}

pub trait Request {
	fn get_peer_addr(&self) -> Option<SocketAddr>;
	fn get_protocol(&self) -> Option<Protocol>;
	fn get_method(&self) -> Option<Method>;
	fn get_connection(&self) -> Option<Connection>;
	fn get_header(&self) -> &Vec<Vec<u8>>;
	fn get_content_length(&self) -> usize;
	fn get_post_data(&self) -> Option<&[u8]>;
	fn get_url(&self) ->Option<&[u8]>;
	fn create_response(&self, contents: Option<Vec<u8>>) -> Response;
}

pub struct Response {
	content: Option<Vec<u8>>,
}

impl Response {
	fn new(contents: Option<Vec<u8>>) -> Response {
		Response {
			content: contents,
		}
	}
}

pub trait Handler {
	fn handle(&self, &Request) -> Response;
}

struct RequestImpl<'a, T: 'a> {
	http_handler: &'a HttpHandler<T>,
	peer_addr: Option<SocketAddr>,
	method: Option<Method>,
	protocol: Option<Protocol>,
	connection: Option<Connection>,
	header: &'a Vec<Vec<u8>>,
	content_length: usize,
	post_data: Option<&'a [u8]>,
}

impl<'a, T: Handler> Request for RequestImpl<'a, T> {
	fn get_peer_addr(&self) -> Option<SocketAddr> {
		self.peer_addr
	}
	fn get_protocol(&self) -> Option<Protocol> {
		self.protocol
	}
	fn get_method(&self) -> Option<Method> {
		self.method
	}
	fn get_connection(&self) -> Option<Connection> {
		None
	}
	fn get_header(&self) -> &Vec<Vec<u8>> {
		self.header
	}
	fn get_content_length(&self) -> usize {
		self.content_length
	}
	fn get_post_data(&self) -> Option<&[u8]> {
		self.post_data
	}
	fn get_url(&self) -> Option<&[u8]> {
		let header = self.get_header();
		if ! header.is_empty() {
			//let ref first_line = header[0];
			let first_line = &header[0];
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
	fn create_response(&self, contents: Option<Vec<u8>>) -> Response {
		Response::new(contents)
	}
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
				if let Some(pos) = self.buffer[..self.offset].iter().position(|&x| x == 10) {
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

	pub fn read_post_data(&mut self, stream: &mut TcpStream, content_length: usize) -> Option<&[u8]> {
		loop {
			if self.offset >= content_length {
				return Some(&self.buffer[..content_length]);
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

		self.offset = 0;
		for i in self.buffer.as_mut().into_iter() {
			*i = 0;
		}

		let peer_addr = stream.peer_addr().ok();

		let mut method: Option<Method> = None;
		let mut protocol: Option<Protocol> = None;
		let mut header: Vec<Vec<u8>> = Vec::new();

		while let Some(line) = self.read_line(&mut stream) {
			if line.len() == 0 { break; }
			if header.is_empty() {
				if let Some(pos) = line.iter().position(|&x| x == 32) {
					if line[..pos].eq_ignore_ascii_case(b"GET") {
						method = Some(Method::GET);
					} else if line[..pos].eq_ignore_ascii_case(b"POST") {
						method = Some(Method::POST);
					}
				}
				if let Some(pos) = line.iter().rposition(|&x| x == 32) {
					if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.0") {
						protocol = Some(Protocol::Http10);
					} else if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.1") {
						protocol = Some(Protocol::Http11);
					}
				}
			}
			header.push(line);
		}

		if method.is_none() {
			let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
			let _ = stream.flush();
			let _ = stream.shutdown(Shutdown::Both);
			return;
		}

		if protocol.is_none() {
			let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
			let _ = stream.flush();
			let _ = stream.shutdown(Shutdown::Both);
			return;
		}

		let mut content_length: usize = 0;
		let mut post_data: Option<&[u8]> = None;
		if let Some(Method::POST) = method {
			for line in &header {
				if line.len() > 15 && line[..15].eq_ignore_ascii_case(b"Content-Length:") {
					content_length = line[15..].iter().fold(0, |a, &x|
						if 48 <= x && x <= 57 { a * 10 + x as usize - 48 } else { a }
					);
				}
			}
			loop {
				if self.offset < content_length && self.offset < self.buffer.len() {
					let size = stream.read(&mut self.buffer[self.offset..]).unwrap();
					if size == 0 { break; }
					self.offset = self.offset + size;
				} else {
					break;
				}
			}
			post_data = Some(&self.buffer[..content_length]);
		}

		let request = RequestImpl {
			http_handler: self,
			peer_addr: peer_addr,
			method: method,
			protocol: protocol,
			connection: None,
			header: &header,
			content_length: content_length,
			post_data: post_data,
		};

		let response = self.handler.handle(&request as &Request);
		if let Some(content) = response.content {
			let mut header = String::new();
			if let Some(ref proto) = protocol {
				match *proto {
					Protocol::Http10 => header.push_str("HTTP/1.0 200 OK\r\n"),
					Protocol::Http11 => header.push_str("HTTP/1.1 200 OK\r\n"),
				}
			}
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

		let _ = stream.flush();
		let _ = stream.shutdown(Shutdown::Both);
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
	}
}
