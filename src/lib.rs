use std::str::from_utf8;
use std::net::SocketAddr;

use http::Header;
use http::Protocol;
use http::Method;
use http::Connection;

pub mod http;
pub mod url;
pub mod html;

pub trait Handler {
	fn handle(&self, &Request) -> Response;
}

pub trait Request {
	fn peer_addr(&self) -> Option<SocketAddr>;
	fn protocol(&self) -> Option<Protocol>;
	fn method(&self) -> Option<Method>;
	fn path(&self) -> Option<&[u8]>;
	fn connection(&self) -> Option<Connection>;
	fn content_length(&self) -> Option<usize>;
	fn post_data(&self) -> Option<&[u8]>;
	fn header(&self) -> &Header;
	fn get_params(&self) -> Params;
	fn post_params(&self) -> Params;
}

pub struct Response {
	content: Option<Vec<u8>>,
	connection: Connection,
	status: u32,
}

impl Response {
	fn new(contents: Option<Vec<u8>>) -> Response {
		Response {
			content: contents,
			connection: Connection::Close,
			status: 200,
		}
	}
	fn connection(&self) -> Connection {
		self.connection
	}
	pub fn from_str(contents: &str) -> Response {
		Response::new(Some(contents.as_bytes().to_vec()))
	}
	pub fn from_string(contents: String) -> Response {
		Response::new(Some(contents.as_bytes().to_vec()))
	}
}

pub struct Param<'a> {
	query: &'a [u8],
}

impl<'a> Param<'a> {
	pub fn name(&self) -> String {
		let mut name = String::new();
		if let Some(pos) = self.query.iter().position(|&x| x == b'=') {
			if let Ok(s) = from_utf8(url::decode_percent(&self.query[0 .. pos]).as_slice()) {
				name.push_str(s);
			}
		}
		name
	}
	pub fn value(&self) -> String {
		let mut value = String::new();
		if let Some(pos) = self.query.iter().position(|&x| x == b'=') {
			if let Ok(s) = from_utf8(url::decode_percent(&self.query[pos + 1 ..]).as_slice()) {
				value.push_str(s);
			}
		} else {
			if let Ok(s) = from_utf8(url::decode_percent(self.query).as_slice()) {
				value.push_str(s);
			}
		}
		value
	}
}

pub struct Params<'a> {
	query: Option<&'a [u8]>,
}

impl<'a> Iterator for Params<'a> {
	type Item = Param<'a>;
	fn next(&mut self) -> Option<Param<'a>> {
		if let Some(q) = self.query {
			if let Some(pos) = q.iter().position(|&x| x == b'&') {
				self.query = Some(&q[pos + 1 ..]);
				Some(Param { query: &q[.. pos] })
			} else {
				self.query = None;
				Some(Param { query: q })
			}
		} else {
			None
		}
	}
}
