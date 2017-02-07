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

const LF: u8 = 10;
const CR: u8 = 13;
const SP: u8 = 32;
const ZERO: u8 = 48;
const NINE: u8 = 57;
const COLON: u8 = 58;
const BUFFER_SIZE: usize = 8192;

fn trim(str: &[u8]) -> &[u8] {
	if let Some(pos1) = str.iter().position(|&x| x != SP) {
		if let Some(pos2) = str.iter().rposition(|&x| x != SP) {
			return &str[pos1 .. pos2 + 1]
		}
	}
	str
}

pub struct Header {
	pub lines: Vec<Vec<u8>>,
}

impl Header {
	fn get_string(&self, name: &[u8]) -> Option<&[u8]> {
		for line in &self.lines {
			if line.len() > name.len() {
				if line[..name.len()].eq_ignore_ascii_case(name) && line[name.len()] == COLON {
					return Some(trim(&line[name.len() + 1..]))
				}
			}
		}
		None
	}
	fn get_number(&self, name: &[u8]) -> Option<usize> {
		if let Some(value) = self.get_string(name) {
			return Some(value.iter().fold(0, |a, &x|
				if ZERO <= x && x <= NINE { a * 10 + (x - ZERO) as usize } else { a }
			));
		}
		None
	}
	pub fn get_protocol(&self) -> Option<Protocol> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos) = line.iter().rposition(|&x| x == SP) {
				if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.0") {
					return Some(Protocol::Http10);
				} else if line[pos + 1..].eq_ignore_ascii_case(b"HTTP/1.1") {
					return Some(Protocol::Http11);
				}
			}
		}
		None
	}
	pub fn get_method(&self) -> Option<Method> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos) = line.iter().position(|&x| x == SP) {
				if line[..pos].eq_ignore_ascii_case(b"GET") {
					return Some(Method::GET);
				} else if line[..pos].eq_ignore_ascii_case(b"POST") {
					return Some(Method::POST);
				}
			}
		}
		None
	}
	pub fn get_path(&self) -> Option<&[u8]> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos1) = line.iter().position(|&x| x == SP) {
				if let Some(pos2) = line.iter().rposition(|&x| x == SP) {
					if pos1 + 1 < pos2 {
						return Some(&line[pos1 + 1..pos2])
					}
				}
			}
		}
		None
	}
	pub fn get_connection(&self) -> Option<Connection> {
		if let Some(value) = self.get_string(b"Connection") {
			if value.eq_ignore_ascii_case(b"keep-alive") {
				return Some(Connection::KeepAlive);
			} else if value.eq_ignore_ascii_case(b"close") {
				return Some(Connection::Close);
			}
		}
		None
	}
	pub fn get_content_length(&self) -> Option<usize> {
		self.get_number(b"Content-Length")
	}
}

struct RequestImpl<'a> {
	peer_addr: Option<SocketAddr>,
	header: &'a Header,
	post_data: Option<&'a [u8]>,
}

impl<'a> Request for RequestImpl<'a> {
	fn get_peer_addr(&self) -> Option<SocketAddr> {
		self.peer_addr
	}
	fn get_protocol(&self) -> Option<Protocol> {
		self.header.get_protocol()
	}
	fn get_method(&self) -> Option<Method> {
		self.header.get_method()
	}
	fn get_path(&self) -> Option<&[u8]> {
		self.header.get_path()
	}
	fn get_connection(&self) -> Option<Connection> {
		self.header.get_connection()
	}
	fn get_content_length(&self) -> Option<usize> {
		self.header.get_content_length()
	}
	fn get_post_data(&self) -> Option<&[u8]> {
		self.post_data
	}
	fn get_header(&self) -> &Header {
		self.header
	}
	fn create_response(&self, contents: Option<Vec<u8>>) -> Response {
		Response::new(contents)
	}
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
				if let Some(pos) = self.buffer[..self.offset].iter().position(|&x| x == LF) {
					let eol = if pos > 0 && self.buffer[pos - 1] == CR { pos - 1 } else { pos };
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

			let mut header_lines: Vec<Vec<u8>> = Vec::new();

			while let Some(line) = self.read_line(&mut stream) {
				if line.len() == 0 { break; }
				header_lines.push(line);
			}

			let header = Header { lines: header_lines };

			if header.get_method().is_none() {
				let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
				let _ = stream.flush();
				let _ = stream.shutdown(Shutdown::Both);
				return;
			}

			if header.get_protocol().is_none() {
				let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
				let _ = stream.flush();
				let _ = stream.shutdown(Shutdown::Both);
				return;
			}

			let mut post_data: Option<&[u8]> = None;
			if let Some(Method::POST) = header.get_method() {
				if let Some(content_length) = header.get_content_length() {
					loop {
						if self.offset < content_length && self.offset < self.buffer.len() {
							let size = stream.read(&mut self.buffer[self.offset..]).unwrap();
							if size == 0 { break; }
							self.offset = self.offset + size;
						} else {
							break;
						}
					}
					post_data = Some(&self.buffer[..self.offset]);
				} else {
					let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
					let _ = stream.flush();
					let _ = stream.shutdown(Shutdown::Both);
					return;
				}
			}

			let request = RequestImpl {
				peer_addr: peer_addr,
				header: &header,
				post_data: post_data,
			};

			let response = self.handler.handle(&request as &Request);

			let mut buf = String::new();

			buf.push_str(match request.get_protocol() {
				Some(Protocol::Http10) => "HTTP/1.0",
				Some(Protocol::Http11) => "HTTP/1.1",
				_ => "HTTP/1.1",
			});

			buf.push_str(" ");
			buf.push_str(match response.status {
				200 => "200 OK",
				_ => "500 Internal Server Error",
			});
			buf.push_str("\r\n");

			let now = time::now_utc();
			let week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
			let month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
			buf.push_str(format!("Date: {}, {} {} {} {:02}:{:02}:{:02} GMT\r\n",
				week.get(now.tm_wday as usize).unwrap(), now.tm_mday,
				month.get(now.tm_mon as usize).unwrap(), 1900 + now.tm_year,
				now.tm_hour, now.tm_min, now.tm_sec).as_str());

			buf.push_str("Server: Akasabi 0.1.0\r\n");

			if let Some(ref content) = response.content {
				buf.push_str("Content-Type: text/html; charset=UTF-8\r\n");
				buf.push_str("Content-Length: ");
				buf.push_str(content.len().to_string().as_str());
				buf.push_str("\r\n");
			}

			buf.push_str("Connection: ");
			buf.push_str(match response.get_connection() {
				Connection::Close => "close",
				Connection::KeepAlive => "keep-alive",
			});
			buf.push_str("\r\n");
			buf.push_str("\r\n");

			let _ = stream.write(buf.as_bytes());

			if let Some(ref content) = response.content {
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
