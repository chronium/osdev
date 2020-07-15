pub mod color;
pub mod css;
pub mod layout;
pub mod painting;
pub mod parser;
pub mod style;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    rc::{Rc, Weak},
    string::String,
    vec,
    vec::Vec,
};

use core::cell::RefCell;

pub type AttrMap = BTreeMap<String, String>;

#[derive(Debug)]
pub struct Node {
    pub parent: RefCell<Vec<Weak<Node>>>,
    pub children: Vec<Rc<Node>>,
    pub node_type: NodeType,
}

#[derive(Debug)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> BTreeSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => BTreeSet::new(),
        }
    }
}

pub fn text(data: String) -> Rc<Node> {
    Rc::new(Node {
        parent: RefCell::new(Vec::new()),
        children: vec![],
        node_type: NodeType::Text(data),
    })
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Rc<Node>>) -> Node {
    Node {
        parent: RefCell::new(Vec::new()),
        children: children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}
