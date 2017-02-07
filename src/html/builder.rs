enum Node<'a> {
	Text(&'a str),
	Tag(Tag<'a>),
}

impl<'a> ToString for Node<'a> {
	fn to_string(&self) -> String {
		match *self {
			Node::Text(s) => String::from(s),
			Node::Tag(ref tag) => tag.to_string(),
		}
	}
}

pub struct Tag<'a> {
	name: &'a str,
	attr: Vec<(&'a str, &'a str)>,
	child: Vec<Node<'a>>,
}

impl<'a> Tag<'a> {
	pub fn new(name: &'a str) -> Tag<'a> {
		Tag {
			name: name,
			attr: Vec::new(),
			child: Vec::new(),
		}
	}
	pub fn push_attr(&mut self, name: &'a str, value: &'a str) {
		self.attr.push((name, value));
	}
	pub fn push_str(&mut self, s: &'a str) {
		self.child.push(Node::Text(s));
	}
	pub fn push_tag(&mut self, tag: Tag<'a>) {
		self.child.push(Node::Tag(tag));
	}
}

impl<'a> ToString for Tag<'a> {
	fn to_string(&self) -> String {
		let mut html = String::new();
		html.push_str("<");
		html.push_str(self.name);
		if ! self.attr.is_empty() {
			html.push_str(" ");
			for &a in &self.attr {
				let (name, value) = a;
				html.push_str(name);
				html.push_str("=\"");
				html.push_str(value);
				html.push_str("\"");
			}
		}
		if self.child.is_empty() {
			html.push_str(" />");
		} else {
			html.push_str(">");
			for node in &self.child {
				html.push_str(node.to_string().as_str());
			}
			html.push_str("</");
			html.push_str(self.name);
			html.push_str(">");
		}
		html
	}
}

pub struct HTML<'a> {
	lang: &'a str,
	pub head: Tag<'a>,
	pub body: Tag<'a>,
}

impl<'a> HTML<'a> {
	pub fn new(title: &'a str, lang: &'a str) -> HTML<'a> {
		HTML {
			lang: lang,
			head: Tag {
				name: "head",
				attr: Vec::new(),
				child: vec![
					Node::Tag(Tag {
						name: "title",
						attr: Vec::new(),
						child: vec![
							Node::Text(title),
						],
					}),
				],
			},
			body: Tag::new("body"),
		}
	}
}

impl<'a> ToString for HTML<'a> {
	fn to_string(&self) -> String {
		let mut html = String::new();
		html.push_str("<!DOCTYPE html>\n");
		html.push_str("<html lang=\"");
		html.push_str(self.lang);
		html.push_str("\">\n");
		html.push_str(self.head.to_string().as_str());
		html.push_str(self.body.to_string().as_str());
		html.push_str("</html>\n");
		html
	}
}
