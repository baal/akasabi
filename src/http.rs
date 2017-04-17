extern crate time;

use std::ascii::AsciiExt;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;

use Handler;
use Request;
use Params;

const LF: u8 = 10;
const CR: u8 = 13;
const SP: u8 = 32;

const BUFFER_SIZE: usize = 8192;
const MAX_POST_SIZE: usize = 65536;

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

pub enum PostData<'a> {
	None,
	Buf(&'a [u8]),
	Vec(Vec<u8>),
}

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
				if line[.. name.len()].eq_ignore_ascii_case(name) && line[name.len()] == b':' {
					return Some(trim(&line[name.len() + 1 ..]))
				}
			}
		}
		None
	}
	fn get_number(&self, name: &[u8]) -> Option<usize> {
		if let Some(value) = self.get_string(name) {
			return Some(value.iter().fold(0, |a, &x|
				if b'0' <= x && x <= b'9' { a * 10 + (x - b'0') as usize } else { a }
			));
		}
		None
	}
	pub fn protocol(&self) -> Option<Protocol> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos) = line.iter().rposition(|&x| x == SP) {
				if line[pos + 1 ..].eq_ignore_ascii_case(b"HTTP/1.0") {
					return Some(Protocol::Http10);
				} else if line[pos + 1 ..].eq_ignore_ascii_case(b"HTTP/1.1") {
					return Some(Protocol::Http11);
				}
			}
		}
		None
	}
	pub fn method(&self) -> Option<Method> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos) = line.iter().position(|&x| x == SP) {
				if line[.. pos].eq_ignore_ascii_case(b"GET") {
					return Some(Method::GET);
				} else if line[.. pos].eq_ignore_ascii_case(b"POST") {
					return Some(Method::POST);
				}
			}
		}
		None
	}
	pub fn path(&self) -> Option<&[u8]> {
		if let Some(line) = self.lines.get(0) {
			if let Some(pos1) = line.iter().position(|&x| x == SP) {
				if let Some(pos2) = line.iter().rposition(|&x| x == SP) {
					if pos1 + 1 < pos2 {
						return Some(&line[pos1 + 1 .. pos2])
					}
				}
			}
		}
		None
	}
	pub fn connection(&self) -> Option<Connection> {
		if let Some(value) = self.get_string(b"Connection") {
			if value.eq_ignore_ascii_case(b"keep-alive") {
				return Some(Connection::KeepAlive);
			} else if value.eq_ignore_ascii_case(b"close") {
				return Some(Connection::Close);
			}
		}
		None
	}
	pub fn content_length(&self) -> Option<usize> {
		self.get_number(b"Content-Length")
	}
}

struct RequestImpl<'a> {
	peer_addr: Option<SocketAddr>,
	header: &'a Header,
	post_data: &'a PostData<'a>,
}

impl<'a> Request for RequestImpl<'a> {
	fn peer_addr(&self) -> Option<SocketAddr> {
		self.peer_addr
	}
	fn protocol(&self) -> Option<Protocol> {
		self.header.protocol()
	}
	fn method(&self) -> Option<Method> {
		self.header.method()
	}
	fn path(&self) -> Option<&[u8]> {
		self.header.path()
	}
	fn connection(&self) -> Option<Connection> {
		self.header.connection()
	}
	fn content_length(&self) -> Option<usize> {
		self.header.content_length()
	}
	fn post_data(&self) -> Option<&[u8]> {
		match *self.post_data {
			PostData::None => Option::None,
			PostData::Buf(slice) => Some(slice),
			PostData::Vec(ref vec) => Some(vec.as_slice()),
		}
	}
	fn header(&self) -> &Header {
		self.header
	}
	fn get_params(&self) -> Params {
		if let Some(path) = self.path() {
			if let Some(pos) = path.iter().position(|&x| x == b'?') {
				return Params { query: Some(&path[pos + 1 ..]) }
			}
		}
		Params { query: None }
	}
	fn post_params(&self) -> Params {
		Params { query: self.post_data() }
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
				if let Some(pos) = self.buffer[.. self.offset].iter().position(|&x| x == LF) {
					let eol = if pos > 0 && self.buffer[pos - 1] == CR { pos - 1 } else { pos };
					let line = self.buffer[0 .. eol].to_vec();
					if pos + 1 < self.offset {
						for i in pos + 1 .. self.offset {
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
				let size = stream.read(&mut self.buffer[self.offset ..]).unwrap();
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

			if header.method().is_none() {
				let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
				let _ = stream.flush();
				let _ = stream.shutdown(Shutdown::Both);
				return;
			}

			if header.protocol().is_none() {
				let _ = stream.write(b"HTTP/1.1 501 Not Implemented\r\n\r\n");
				let _ = stream.flush();
				let _ = stream.shutdown(Shutdown::Both);
				return;
			}

			let mut post_data: PostData = PostData::None;
			if let Some(Method::POST) = header.method() {
				if let Some(length) = header.content_length() {
					if length <= BUFFER_SIZE {
						while self.offset < length && self.offset < self.buffer.len() {
							let size = stream.read(&mut self.buffer[self.offset ..]).unwrap();
							self.offset = self.offset + size;
						}
						post_data = PostData::Buf(&self.buffer[0 .. self.offset]);
					} else if length <= MAX_POST_SIZE {
						let mut large_buffer: Vec<u8> = Vec::with_capacity(length);
						for i in 0 .. self.offset {
							large_buffer.push(self.buffer[i]);
						}
						self.offset = 0;
						while large_buffer.len() < length {
							let size = stream.read(&mut self.buffer).unwrap();
							for i in 0 .. size {
								large_buffer.push(self.buffer[i]);
							}
						}
						post_data = PostData::Vec(large_buffer);
					}
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
				post_data: &post_data,
			};

			let response = self.handler.handle(&request as &Request);

			let mut buf = String::new();

			buf.push_str(match request.protocol() {
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

			buf.push_str("Server: Akasabi 0.1.0 (Rust 1.16.0)\r\n");

			if let Some(ref content) = response.content {
				buf.push_str("Content-Type: text/html; charset=UTF-8\r\n");
				buf.push_str("Content-Length: ");
				buf.push_str(content.len().to_string().as_str());
				buf.push_str("\r\n");
			}

			buf.push_str("Connection: ");
			buf.push_str(match response.connection() {
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
