
use tree;

// pub struct Button<S> {
//     text: String,
//     click: S,
// }

// pub type Value<S> = (Kind, Attributes<S>);
pub type Attributes<S> = Vec<Attribute<S>>;

#[derive(Clone, Debug)]
pub struct Object<S> {
    pub kind: Kind,
    pub attributes: Attributes<S>,

    pub children: Vec<Object<S>>,
}

impl<T: PartialEq> tree::Vertex for Object<T> {
    fn children(&self) -> &[Self] {
        &self.children
    }

    fn compare(&self, other: &Self) -> Option<tree::Difference> {
        if self.kind != other.kind {
            Some(tree::Difference::Kind)
        } else if self.attributes != other.attributes {
            Some(tree::Difference::Value)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Kind {
    Stack,
    Button,
    Label,
    Field,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Attribute<S> {
    Text(String),
    Click(S),
    Placeholder(String),
    Change(fn(String) -> S),
}

pub type Change = tree::Change;
pub type Changeset = tree::Changeset;

pub fn diff<S: PartialEq>(old: &Object<S>, new: &Object<S>) -> Changeset {
    tree::diff(old, new)
}

/// 'Builder' methods for Object
impl<S> Object<S> {
    pub fn text(mut self, text: String) -> Self {
        self.attributes.push(Attribute::Text(text.into()));
        self
    }
    pub fn click(mut self, action: S) -> Self {
        self.attributes.push(Attribute::Click(action));
        self
    }
    pub fn placeholder(mut self, text: String) -> Self {
        self.attributes.push(Attribute::Placeholder(text));
        self
    }
    pub fn change(mut self, messenger: fn(String) -> S) -> Self {
        self.attributes.push(Attribute::Change(messenger));
        self
    }
}

pub fn stack<S>(objects: Vec<Object<S>>) -> Object<S> {
    Object {
        kind: Kind::Stack,
        attributes: vec![],
        children: objects,
    }
}

pub fn label<S>() -> Object<S> {
    Object {
        kind: Kind::Label,
        attributes: vec![],
        children: vec![],
    }
}

pub fn button<S>() -> Object<S> {
    Object {
        kind: Kind::Button,
        attributes: vec![],
        children: vec![],
    }
}

pub fn field<S>() -> Object<S> {
    Object {
        kind: Kind::Field,
        attributes: vec![],
        children: vec![],
    }
}
