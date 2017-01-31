extern crate time;

use std::ascii::AsciiExt;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;

use Handler;
use Request;
use Response;

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

struct RequestImpl<'a> {
	peer_addr: Option<SocketAddr>,
	method: Option<Method>,
	protocol: Option<Protocol>,
	connection: Option<Connection>,
	header: &'a Vec<Vec<u8>>,
	content_length: Option<usize>,
	post_data: Option<&'a [u8]>,
}

impl<'a> Request for RequestImpl<'a> {
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
		self.connection
	}
	fn get_header(&self) -> &Vec<Vec<u8>> {
		self.header
	}
	fn get_content_length(&self) -> Option<usize> {
		self.content_length
	}
	fn get_post_data(&self) -> Option<&[u8]> {
		self.post_data
	}
	fn get_url(&self) -> Option<&[u8]> {
		let header = self.get_header();
		if ! header.is_empty() {
			if let Some(initial_request_line) = header.get(0) {
				if let Some(pos1) = initial_request_line.iter().position(|&x| x == 32) {
					if let Some(pos2) = initial_request_line.iter().rposition(|&x| x == 32) {
						if pos1 + 1 < pos2 {
							return Some(&initial_request_line[pos1 + 1..pos2])
						}
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

const BUFFER_SIZE: usize = 8192;

fn trim(str: &[u8]) -> &[u8] {
	if let Some(pos1) = str.iter().position(|&x| x != 32) {
		if let Some(pos2) = str.iter().rposition(|&x| x != 32) {
			return &str[pos1 .. pos2 + 1]
		}
	}
	str
}

pub struct HttpHandler<T> {
	handler: T,
	offset: usize,
	buffer: [u8; BUFFER_SIZE],
}

impl<T: Handler> HttpHandler<T> {

	pub fn new(h: T) -> HttpHandler<T> {
		HttpHandler {
			handler: h,
			offset: 0,
			buffer: [0; BUFFER_SIZE],
		}
	}

	fn read_line(&mut self, stream: &mut TcpStream) -> Option<Vec<u8>> {
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

	pub fn handle(&mut self, mut stream: TcpStream) {

		loop {
			self.offset = 0;
			for i in self.buffer.as_mut().into_iter() {
				*i = 0;
			}

			let peer_addr = stream.peer_addr().ok();

			let mut method: Option<Method> = None;
			let mut protocol: Option<Protocol> = None;
			let mut connection: Option<Connection> = None;
			let mut header: Vec<Vec<u8>> = Vec::new();

			while let Some(line) = self.read_line(&mut stream) {
				if line.len() == 0 { break; }
				header.push(line);
			}

			if ! header.is_empty() {
				let line = &header[0];
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

			for line in &header {
				let prefix = b"Connection:";
				let prefix_len = prefix.len();
				if line.len() > prefix_len && line[..prefix_len].eq_ignore_ascii_case(prefix) {
					let value = trim(&line[prefix_len..]);
					if value.eq_ignore_ascii_case(b"keep-alive") {
						connection = Some(Connection::KeepAlive);
					} else if value.eq_ignore_ascii_case(b"close") {
						connection = Some(Connection::Close);
					}
				}
			}

			let mut content_length: Option<usize> = None;
			let mut post_data: Option<&[u8]> = None;
			if let Some(Method::POST) = method {
				for line in &header {
					if line.len() > 15 && line[..15].eq_ignore_ascii_case(b"Content-Length:") {
						content_length = Some(line[15..].iter().fold(0, |a, &x|
							if 48 <= x && x <= 57 { a * 10 + x as usize - 48 } else { a }
						));
					}
				}
				if let Some(len) = content_length {
					loop {
						if self.offset < len && self.offset < self.buffer.len() {
							let size = stream.read(&mut self.buffer[self.offset..]).unwrap();
							if size == 0 { break; }
							self.offset = self.offset + size;
						} else {
							break;
						}
					}
					post_data = Some(&self.buffer[..len]);
				} else {
					let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
					let _ = stream.flush();
					let _ = stream.shutdown(Shutdown::Both);
					return;
				}
			}

			let request = RequestImpl {
				peer_addr: peer_addr,
				method: method,
				protocol: protocol,
				connection: connection,
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
				let now = time::now_utc();
				let week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
				let month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
				header.push_str(format!("Date: {}, {} {} {} {:02}:{:02}:{:02} GMT\r\n",
					week.get(now.tm_wday as usize).unwrap(), now.tm_mday,
					month.get(now.tm_mon as usize).unwrap(), 1900 + now.tm_year,
					now.tm_hour, now.tm_min, now.tm_sec).as_str());
				header.push_str("Server: Rust 1.13.0\r\n");
				header.push_str("Content-Type: text/html; charset=UTF-8\r\n");
				header.push_str("Content-Length: ");
				header.push_str(content.len().to_string().as_str());
				header.push_str("\r\n");
				match response.connection {
					Connection::Close => header.push_str("Connection: close\r\n"),
					Connection::KeepAlive => header.push_str("Connection: keep-alive\r\n"),
				}
				header.push_str("\r\n");
				let _ = stream.write(header.as_bytes());
				let _ = stream.write(content.as_slice());
			}

			if let Connection::Close = response.connection {
				let _ = stream.flush();
				let _ = stream.shutdown(Shutdown::Both);
				break;
			}
		}
	}
}
