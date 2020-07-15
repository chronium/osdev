use super::{
    css::{Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value},
    layout::Display,
    parser::css,
    ElementData, Node, NodeType,
};

use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

pub type PropertyMap = BTreeMap<String, Value>;

static NONE_DISPLAY: [&'static str; 4] = ["head", "meta", "title", "style"];
static DEFAULT_BLOCK: [&'static str; 11] = [
    "address",
    "blockquote",
    "dd",
    "div",
    "dl",
    "form",
    "p",
    "ul",
    "h1",
    "html",
    "body",
];
static DEFAULT_INHERIT: [&'static str; 3] = ["color", "font-size", "line-height"];

#[derive(Debug)]
pub struct StyledNode<'a> {
    node: &'a Node,
    specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl StyledNode<'_> {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn tag_name(&self) -> String {
        match self.node.node_type {
            NodeType::Element(ref data) => data.tag_name.clone(),
            NodeType::Text(ref string) => {
                let mut text = "text: ".to_string();
                if string.len() > 3 {
                    text.push_str(&string.as_str()[0..=3]);
                } else {
                    text.push_str(string.as_str());
                }
                text
            }
        }
    }

    pub fn check_none_diplay_node(&mut self) {
        if NONE_DISPLAY.contains(&self.tag_name().as_str()) {
            self.specified_values
                .insert("display".to_string(), Value::Keyword("none".to_string()));
        };
    }
}

fn matches(node: Rc<Node>, elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, &simple_selector),
        Selector::Descendant(ref descendant_selector) => {
            matches_descendant_selector(node, elem, descendant_selector.as_slice())
        }
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(&**class))
    {
        return false;
    }

    return true;
}

fn matches_descendant_selector(
    node: Rc<Node>,
    elem: &ElementData,
    selector: &[SimpleSelector],
) -> bool {
    assert!(selector.len() > 1);

    if !matches_simple_selector(elem, selector.last().unwrap()) {
        return false;
    }

    let current_selector = &selector[0..selector.len()];
    return matches_ancestor(node, current_selector);
}

fn matches_ancestor(node: Rc<Node>, selector: &[SimpleSelector]) -> bool {
    let mut current_node = node;
    let mut matching_node: Option<Rc<Node>> = None;
    loop {
        match get_parent(&current_node) {
            Some(parent_node) => {
                if let NodeType::Element(ref parent_elem) = parent_node.node_type {
                    if matches_simple_selector(parent_elem, selector.last().unwrap()) {
                        matching_node = Some(parent_node.clone());
                        break;
                    }
                    current_node = parent_node.clone();
                }
            }
            None => break,
        }
    }

    match matching_node {
        Some(_) => {
            if selector.len() == 1 {
                return true;
            }
        }
        None => return false,
    }

    return matches_ancestor(matching_node.unwrap(), &selector[0..selector.len()]);
}

fn get_parent(node: &Rc<Node>) -> Option<Rc<Node>> {
    if node.parent.borrow().is_empty() {
        return None;
    }
    node.parent.borrow().last().unwrap().upgrade()
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn match_rule<'a>(node: Rc<Node>, elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(node.clone(), elem, *selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matching_rules<'a>(
    node: Rc<Node>,
    elem: &ElementData,
    stylesheet: &'a Stylesheet,
) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(node.clone(), elem, rule))
        .collect()
}

fn specified_values(
    node: Rc<Node>,
    elem: &ElementData,
    stylesheet: &Stylesheet,
    inherits: &PropertyMap,
) -> PropertyMap {
    let mut values = BTreeMap::new();
    let mut rules = matching_rules(node, elem, stylesheet);

    // Go through the rules from lowest to highest specificity.
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for &(_, rule) in rules.iter() {
        for declaration in rule.declarations.iter() {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    apply_inline_style(&mut values, elem);
    apply_inherit_style(&mut values, inherits);
    return values;
}

fn apply_inherit_style(values: &mut PropertyMap, inherits: &PropertyMap) {
    for (name, value) in inherits.iter() {
        if let None = values.get(name) {
            values.insert(name.clone(), value.clone());
        };
    }
}

fn get_inherit_style(values: &PropertyMap) -> PropertyMap {
    let mut inherits = BTreeMap::new();
    for (name, value) in values.iter() {
        if DEFAULT_INHERIT.contains(&name.as_str()) {
            inherits.insert(name.clone(), value.clone());
        }
    }
    inherits
}

fn apply_inline_style(values: &mut PropertyMap, elem: &ElementData) {
    if let Some(style_string) = elem.attributes.get("style") {
        let mut last_idx;
        let mut source = style_string.clone();
        if source.chars().last().unwrap() != ';' {
            last_idx = source.len();
            source.insert(last_idx, ';');
        }
        source.insert(0, '{');
        last_idx = source.len();
        source.insert(last_idx, '}');

        /*for decl in css::parse_inline_style(source).into_iter() {
            values.insert(decl.name, decl.value);
        }*/
    }
}

pub fn style_tree<'a>(
    root: &'a Rc<Node>,
    stylesheet: &'a Stylesheet,
    inherits: &PropertyMap,
) -> StyledNode<'a> {
    let values = match root.node_type {
        NodeType::Element(ref elem) => specified_values(root.clone(), elem, stylesheet, inherits),
        NodeType::Text(_) => BTreeMap::new(),
    };
    let new_inherits = get_inherit_style(&values);

    let mut new_style_node = StyledNode {
        node: &root,
        specified_values: values,
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet, &new_inherits))
            .collect(),
    };

    new_style_node.check_none_diplay_node();
    new_style_node
}
