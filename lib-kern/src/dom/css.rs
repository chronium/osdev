use alloc::{string::String, vec::Vec};

use super::color::Color;

pub type Specificity = (usize, usize, usize);

#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
    Descendant(DescendantSelector),
}

pub type DescendantSelector = Vec<SimpleSelector>;

impl Selector {
    pub fn specificity(&self) -> Specificity {
        match *self {
            Selector::Simple(ref simple) => {
                let a = simple.id.iter().len();
                let b = simple.class.len();
                let c = simple.tag_name.iter().len();
                return (a, b, c);
            }
            Selector::Descendant(ref descendant) => {
                let mut specificity = (0, 0, 0);
                for i in descendant.iter() {
                    specificity.0 += i.id.iter().len();
                    specificity.1 += i.class.len();
                    specificity.2 += i.tag_name.iter().len();
                }
                return specificity;
            }
        }
    }
}

#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

impl Value {
    pub fn to_px(&self) -> Option<f32> {
        match *self {
            Value::Length(f, Unit::Px) => Some(f),
            Value::Length(f, Unit::Em) => Some(f * FONT_SIZE),
            Value::Length(_, Unit::Percent) => None,
            _ => Some(0f32),
        }
    }

    pub fn percent_to_px(&self, container_width: f32) -> f32 {
        if let Value::Length(f, Unit::Percent) = *self {
            return container_width * f / 100f32;
        } else {
            return self.to_px().unwrap();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Percent,
    Default,
}

static FONT_SIZE: f32 = 12.0;
