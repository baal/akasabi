extern crate akasabi;

use std::net::TcpListener;

use akasabi::http::HttpHandler;
use akasabi::http::Method;
use akasabi::Handler;
use akasabi::Request;
use akasabi::Response;

use akasabi::html;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, req: &Request) -> Response {
		let mut html = String::new();
		html.push_str("<!DOCTYPE html>\n");
		html.push_str("<html lang=\"ja\">\n");
		html.push_str("<head>\n");
		html.push_str("<title>POST</title>\n");
		html.push_str("</head>\n");
		html.push_str("<body>\n");
		if let Some(Method::POST) = req.method() {
			html.push_str("<ul>\n");
			for param in req.post_params() {
					html.push_str("<li>&quot;");
					html.push_str(html::escape_html(param.name().as_str()).as_str());
					html.push_str("&quot;=&quot;");
					html.push_str(html::escape_html(param.value().as_str()).as_str());
					html.push_str("&quot;</li>\n");
			}
			html.push_str("</ul>\n");
		}
		html.push_str("<form method=\"POST\">\n");
		html.push_str("<input type=\"text\" name=\"test1\" value=\"");
		if let Some(test1) = req.post_params().find(|&ref x| x.name() == "test1") {
			html.push_str(html::escape_html(test1.value().as_str()).as_str());
		}
		html.push_str("\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test2\" value=\"");
		if let Some(test2) = req.post_params().find(|&ref x| x.name() == "test2") {
			html.push_str(html::escape_html(test2.value().as_str()).as_str());
		}
		html.push_str("\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test3\" value=\"");
		if let Some(test3) = req.post_params().find(|&ref x| x.name() == "test3") {
			html.push_str(html::escape_html(test3.value().as_str()).as_str());
		}
		html.push_str("\" /><br />\n");
		html.push_str("<button type=\"submit\">SUBMIT</button><br />\n");
		html.push_str("</form>\n");
		html.push_str("</body>\n");
		html.push_str("</html>\n");
		Response::from_string(html)
	}
}

fn main() {
	let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
	for stream in listener.incoming() {
		HttpHandler::new(MyHandler).handle(stream.unwrap());
	}
}
