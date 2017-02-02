extern crate rustweb;

use std::str;
use std::net::TcpListener;

use rustweb::http::Protocol;
use rustweb::http::Method;
use rustweb::http::Connection;

use rustweb::http::HttpHandler;
use rustweb::Handler;
use rustweb::Request;
use rustweb::Response;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, req: &Request) -> Response {
		if let Some(addr) = req.get_peer_addr() {
			println!("remote_addr=\"{}\"", addr);
		}
		if let Some(protocol) = req.get_protocol() {
			println!("protocol=\"{}\"", match protocol {
				Protocol::Http10 => "Http/1.0",
				Protocol::Http11 => "Http/1.1",
			});
		}
		if let Some(method) = req.get_method() {
			println!("method=\"{}\"", match method {
				Method::GET => "GET",
				Method::POST => "POST",
			});
		}
		if let Some(path) = req.get_path() {
			println!("path=\"{}\"", str::from_utf8(path).unwrap());
		}
		if let Some(connection) = req.get_connection() {
			println!("connection=\"{}\"", match connection {
				Connection::Close => "close",
				Connection::KeepAlive => "keep-alive",
			});
		}
		if let Some(content_length) = req.get_content_length() {
			println!("content_length=\"{}\"", content_length);
		}
		if let Some(post_data) = req.get_post_data() {
			println!("post_data=\"{}\"", str::from_utf8(post_data).unwrap());
		}
		let header = req.get_header();
		for line in &header.lines {
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
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		HttpHandler::new(MyHandler).handle(stream.unwrap());
	}
}
