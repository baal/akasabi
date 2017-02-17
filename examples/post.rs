extern crate akasabi;

use std::str;
use std::net::TcpListener;

use akasabi::http::HttpHandler;
use akasabi::Handler;
use akasabi::Request;
use akasabi::Response;

use akasabi::html;
use akasabi::url;

struct MyHandler;

impl Handler for MyHandler {
	fn handle(&self, req: &Request) -> Response {
		let mut html = String::new();
		html.push_str("<!DOCTYPE html>\n");
		html.push_str("<html lang=\"ja\">\n");
		html.push_str("<head>\n");
		html.push_str("<title>GET</title>\n");
		html.push_str("</head>\n");
		html.push_str("<body>\n");
		if let Some(post) = req.get_post_data() {
			html.push_str("<ul>\n");
			for pair in post.split(|&x| x == b'&') {
				html.push_str("<li>&quot;");
				if let Some(pos) = pair.iter().position(|&x| x == b'=') {
					if let Ok(name) = str::from_utf8(url::decode_percent(&pair[..pos]).as_slice()) {
						html.push_str(html::escape_html(name).as_str());
					}
					html.push_str("&quot;=&quot;");
					if let Ok(value) = str::from_utf8(url::decode_percent(&pair[pos + 1..]).as_slice()) {
						html.push_str(html::escape_html(value).as_str());
					}
				} else {
					if let Ok(s) = str::from_utf8(url::decode_percent(pair).as_slice()) {
						html.push_str(html::escape_html(s).as_str());
					}
				}
				html.push_str("&quot;</li>\n");
			}
			html.push_str("</ul>\n");
		}
		html.push_str("<form method=\"POST\">\n");
		html.push_str("<input type=\"text\" name=\"test1\" value=\"\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test2\" value=\"\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test3\" value=\"\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test4\" value=\"\" /><br />\n");
		html.push_str("<input type=\"text\" name=\"test5\" value=\"\" /><br />\n");
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
