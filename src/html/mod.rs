pub mod builder;

pub fn escape_html(s: &str) -> String {
	let mut result = String::new();
	for c in s.chars() {
		match c {
			'<' => { result.push_str("&lt;"); },
			'>' => { result.push_str("&gt;"); },
			'"' => { result.push_str("&quot;"); },
			'&' => { result.push_str("&amp;"); },
			_ => { result.push(c); },
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::escape_html;
	#[test]
	fn test_escape_html() {
		assert_eq!("&lt;&gt;&quot;&amp;", escape_html("<>\"&"));
	}
}
