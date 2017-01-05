extern crate rustweb;

use std::str;
use std::net::TcpListener;

use rustweb::Handler;
use rustweb::HttpHandler;
use rustweb::Request;
use rustweb::Response;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, req: &Request, res: &mut Response) {
		println!("remote_addr=\"{}\"", req.remote_addr.unwrap());
		if let Some(ref uri) = req.uri {
			println!("uri=\"{}\"", str::from_utf8(&uri).unwrap());
		}
		for line in &req.header {
			println!("\"{}\"", str::from_utf8(&line).unwrap());
		}
		res.content.push_str("<!DOCTYPE html>\n");
		res.content.push_str("<html lang=\"ja\">\n");
		res.content.push_str("<head><title>TEST</title></head>\n");
		res.content.push_str("<body>\n");
		res.content.push_str("<form method=\"POST\">\n");
		res.content.push_str("<input type=\"text\" name=\"test\" />\n");
		res.content.push_str("<button>submit</button>\n");
		res.content.push_str("</form>\n");
		res.content.push_str("</body>\n");
		res.content.push_str("</html>\n");
	}
}

fn main() {
	let handler = HttpHandler::new(MyHandler);
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		handler.handle(stream.unwrap());
	}
}
