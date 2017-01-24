use std::net::SocketAddr;

use http::Protocol;
use http::Method;
use http::Connection;

pub mod http;

pub trait Handler {
	fn handle(&self, &Request) -> Response;
}

pub trait Request {
	fn get_peer_addr(&self) -> Option<SocketAddr>;
	fn get_protocol(&self) -> Option<Protocol>;
	fn get_method(&self) -> Option<Method>;
	fn get_connection(&self) -> Option<Connection>;
	fn get_header(&self) -> &Vec<Vec<u8>>;
	fn get_content_length(&self) -> Option<usize>;
	fn get_post_data(&self) -> Option<&[u8]>;
	fn get_url(&self) ->Option<&[u8]>;
	fn create_response(&self, contents: Option<Vec<u8>>) -> Response;
}

pub struct Response {
	content: Option<Vec<u8>>,
	connection: Connection,
}

impl Response {
	fn new(contents: Option<Vec<u8>>) -> Response {
		Response {
			content: contents,
			connection: Connection::Close,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
	}
}
