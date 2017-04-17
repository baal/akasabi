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
		if let Some(addr) = req.peer_addr() {
			println!("remote_addr=\"{}\"", addr);
		}
		if let Some(protocol) = req.protocol() {
			println!("protocol=\"{}\"", match protocol {
				Protocol::Http10 => "Http/1.0",
				Protocol::Http11 => "Http/1.1",
			});
		}
		if let Some(method) = req.method() {
			println!("method=\"{}\"", match method {
				Method::GET => "GET",
				Method::POST => "POST",
			});
		}
		if let Some(path) = req.path() {
			println!("path=\"{}\"", str::from_utf8(path).unwrap());
		}
		if let Some(connection) = req.connection() {
			println!("connection=\"{}\"", match connection {
				Connection::Close => "close",
				Connection::KeepAlive => "keep-alive",
			});
		}
		if let Some(content_length) = req.content_length() {
			println!("content_length=\"{}\"", content_length);
		}
		if let Some(post_data) = req.post_data() {
			println!("post_data=\"{}\"", str::from_utf8(post_data).unwrap());
		}
		let header = req.header();
		for line in &header.lines {
			println!("\"{}\"", str::from_utf8(line.as_slice()).unwrap());
		}
		let mut html = HTML::new("akasabi", "ja");
		let mut div = Tag::new("h1");
		div.push_escape("It works!");
		html.body.push_tag(div);
		Response::from_string(html.to_string())
	}
}

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		HttpHandler::new(MyHandler).handle(stream.unwrap());
	}
}
