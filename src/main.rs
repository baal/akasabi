extern crate akasabi;

use std::str;
use std::net::TcpListener;

use akasabi::http::Protocol;
use akasabi::http::Method;
use akasabi::http::Connection;

use akasabi::http::HttpHandler;
use akasabi::Handler;
use akasabi::Request;
use akasabi::Response;

use akasabi::html::builder::HTML;
use akasabi::html::builder::Tag;

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
		let mut html = HTML::new("TEST", "ja");
		let mut div = Tag::new("div");
		div.push_str("TEST");
		html.body.push_tag(div);
		req.create_response(Some(html.to_string().as_bytes().to_vec()))
	}
}

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		HttpHandler::new(MyHandler).handle(stream.unwrap());
	}
}
