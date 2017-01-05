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
	pub uri: Option<Vec<u8>>,
	pub connection: Connection,
	pub header: Vec<Vec<u8>>,
}

impl Request {
	fn new(stream: &TcpStream) -> Request {
		Request {
			remote_addr: stream.peer_addr().ok(),
			protocol: None,
			method: None,
			uri: None,
			connection: Connection::Close,
			header: Vec::new(),
		}
	}
}

pub struct Response {
	pub content: String,
}

impl Response {
	fn new() -> Response {
		Response {
			content: String::new(),
		}
	}
}

pub trait Handler {
	fn handle(&self, &Request, &mut Response);
}

pub struct HttpHandler<T> {
	handler: T,
}

impl<T: Handler> HttpHandler<T> {

	pub fn new(h: T) -> HttpHandler<T> {
		HttpHandler {
			handler: h,
		}
	}

	pub fn handle(&self, mut stream: TcpStream) {
		loop {
			let mut offset = 0;
			let mut buf: [u8; 8192] = [0; 8192];

			let mut request = Request::new(&stream);
			let mut response = Response::new();

			let mut flag_read_request = false;
			let mut flag_read_header = false;

			'read_loop: while offset < buf.len() {
				let mut size = stream.read(&mut buf[offset..]).unwrap();
				if size == 0 { break; }
				if ! flag_read_header {
					while let Some(pos) = buf[offset..offset + size].iter().position(|&x| x == 10) {
						let eol = if offset + pos > 0 && buf[offset + pos - 1] == 13 { pos - 1 } else { pos };
						if offset + eol > 0 {
							if ! flag_read_request {
								if let Some(pos1) = buf[..offset + eol].iter().position(|&x| x == 32) {
									if buf[..pos1].eq_ignore_ascii_case(b"GET") {
										request.method = Some(Method::Get);
									} else if buf[..pos1].eq_ignore_ascii_case(b"POST") {
										request.method = Some(Method::Post);
									}
									if let Some(pos2) = buf[pos1 + 1..offset + eol].iter().position(|&x| x == 32) {
										if buf[pos2 + 1..offset + eol].eq_ignore_ascii_case(b"HTTP/1.0") {
											request.protocol = Some(Protocol::Http10);
										} else if buf[pos2 + 1..offset + eol].eq_ignore_ascii_case(b"HTTP/1.1") {
											request.protocol = Some(Protocol::Http11);
										}
										request.uri = Some(buf[pos1 + 1..pos1 + 1 + pos2].to_vec());
									}
								}
								flag_read_request = true;
							}
							request.header.push(buf[..offset + eol].to_vec());
						} else {
							flag_read_header = true;
						}
						if size > pos + 1 {
							for i in pos + 1..size {
								buf[i - pos - 1] = buf[offset + i];
							}
							offset = 0;
							size = size - pos - 1;
						}
						if flag_read_header {
							break 'read_loop;
						}
					}
				}
				offset = offset + size;
			}

			self.handler.handle(&request, &mut response);

			let mut header = String::new();
			header.push_str("HTTP/1.1 200 OK\r\n");
			header.push_str("Server: Rust 1.13.0\r\n");
			header.push_str("Content-Type: text/html; charset=UTF-8\r\n");
			header.push_str("Content-Length: ");
			header.push_str(response.content.len().to_string().as_str());
			header.push_str("\r\n");
			header.push_str("Connection: keep-alive\r\n");
			header.push_str("\r\n");
			let _ = stream.write(header.as_bytes());
			let _ = stream.write(response.content.as_bytes());
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
	}
}
