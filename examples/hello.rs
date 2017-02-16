extern crate akasabi;

use std::net::TcpListener;

use akasabi::http::HttpHandler;
use akasabi::Handler;
use akasabi::Request;
use akasabi::Response;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, _: &Request) -> Response {
		Response::from_str("Hello, world!")
	}
}

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		HttpHandler::new(MyHandler).handle(stream.unwrap());
	}
}
