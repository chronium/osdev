use crate::dom;

use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};

pub fn parse(source: String) -> Rc<dom::Node> {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        Rc::new(dom::elem("html".to_string(), BTreeMap::new(), nodes))
    }
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn parse_nodes(&mut self) -> Vec<Rc<dom::Node>> {
        let mut nodes = vec![];

        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }

        nodes
    }

    fn parse_node(&mut self) -> Rc<dom::Node> {
        let node = match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        };

        for child in node.children.iter() {
            child.parent.borrow_mut().push(Rc::downgrade(&node));
        }

        node
    }

    fn parse_element(&mut self) -> Rc<dom::Node> {
        self.consume_comment();

        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        if self.is_self_closing_tag(tag_name.as_str()) {
            return Rc::new(dom::elem(tag_name, attrs, Vec::new()));
        }

        let children = self.parse_nodes();

        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        Rc::new(dom::elem(tag_name, attrs, children))
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => true,
            _ => false,
        })
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = BTreeMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    fn parse_text(&mut self) -> Rc<dom::Node> {
        dom::text(self.consume_while(|c| c != '<'))
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, test: F) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        return result;
    }

    fn consume_char(&mut self) -> char {
        self.pos += 1;
        self.input.chars().nth(self.pos - 1).unwrap()
    }

    fn next_char(&self) -> char {
        self.input.chars().nth(self.pos).unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input.get(self.pos..).unwrap().starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_comment(&mut self) {
        while self.starts_with("<!") {
            assert!(self.consume_char() == '<');
            assert!(self.consume_char() == '!');
            self.consume_while(|c| c != '>');
            assert!(self.consume_char() == '>');
            self.consume_whitespace();
        }
    }

    fn is_self_closing_tag(&self, tag_name: &str) -> bool {
        let self_closing_tags = vec!["input", "meta"];
        self_closing_tags.contains(&tag_name)
    }
}
