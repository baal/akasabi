extern crate rustweb;

use std::str;
use std::string::String;
use std::net::TcpListener;

use rustweb::Handler;
use rustweb::HttpHandler;
use rustweb::Request;
use rustweb::Response;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, req: &Request) -> Response {
		if let Some(addr) = req.get_peer_addr() {
			println!("remote_addr=\"{}\"", addr);
		}
		if let Some(url) = req.get_url() {
			println!("url=\"{}\"", str::from_utf8(url).unwrap());
		}
		for line in req.get_header() {
			println!("\"{}\"", str::from_utf8(line.as_slice()).unwrap());
		}
		let mut content = String::new();
		content.push_str("<!DOCTYPE html>\n");
		content.push_str("<html lang=\"ja\">\n");
		content.push_str("<head><title>TEST</title></head>\n");
		content.push_str("<body>\n");
		content.push_str("<form method=\"POST\">\n");
		content.push_str("<input type=\"text\" name=\"test\" />\n");
		content.push_str("<button>submit</button>\n");
		content.push_str("</form>\n");
		content.push_str("</body>\n");
		content.push_str("</html>\n");
		req.create_response(Some(content.as_bytes().to_vec()))
	}
}

fn main() {
	let mut handler = HttpHandler::new(MyHandler);
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		handler.handle(stream.unwrap());
	}
}
