use std::cell::RefCell;
use std::rc::Rc;

pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub children: RefCell<Vec<NodeRef>>,
    pub props: Option<Props>,
}

#[derive(Clone, Debug)]
pub struct Props {
    pub created: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub doc_type: Option<String>,
    pub owner_node: Option<NodeRef>,
    pub meta_node: Option<NodeRef>,
    pub source_node: Option<NodeRef>,
}

#[derive(Clone, Debug)]
pub enum Meta {
    // // SYS_A11
    // Color,
    // // SYS_A12
    // Locked,
    // SYS_A13
    SuperTags(Vec<String>),
    // // SYS_A14
    // ChildSuperTag,
    // SYS_A15
    SearchExpression,
}

#[derive(Clone, Debug)]
pub enum DocType {
    Text,
    Codeblock,
    Search,
    Tuple,
}

impl Node {
    pub fn new(id: String, props: Option<Props>, children: RefCell<Vec<NodeRef>>) -> Node {
        Node {
            id,
            props,
            children,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.get_props().name.clone()
    }

    pub fn get_props(&self) -> Props {
        if self.props.is_none() {
            dbg!(&self.id);
        }
        self.props.clone().unwrap()
    }

    pub fn get_children(&self) -> Vec<NodeRef> {
        self.children
            .borrow()
            .iter()
            .map(|child| Rc::clone(child))
            .collect()
    }

    pub fn get_doc_type(&self) -> DocType {
        let doc_type = self.get_props().doc_type;

        match doc_type.unwrap_or(String::from("")).as_str() {
            "tuple" => DocType::Tuple,
            "codeblock" => DocType::Codeblock,
            "search" => DocType::Search,
            _ => DocType::Text,
        }
    }

    pub fn get_owner_node(&self) -> Option<NodeRef> {
        self.get_props().owner_node
    }

    pub fn get_tag_list(&self) -> Vec<String> {
        for meta in self.get_meta() {
            if let Meta::SuperTags(tags) = meta {
                return tags;
            }
        }
        return vec![];
    }

    pub fn get_meta_node(&self) -> Option<NodeRef> {
        self.get_props().meta_node
    }

    pub fn get_meta(&self) -> Vec<Meta> {
        let meta_node = self.get_meta_node();
        if meta_node.is_none() {
            return vec![];
        }
        let meta_node = meta_node.unwrap();
        let meta_node = meta_node.borrow();

        meta_node
            .get_children()
            .iter()
            .map(|item| Meta::from_node(Rc::clone(item)))
            .filter(|meta| meta.is_some())
            .map(|meta| meta.unwrap())
            .collect::<Vec<Meta>>()
    }

    pub fn is_in_trash(&self) -> bool {
        if self.id.ends_with("_TRASH") {
            return true;
        }

        let mut owner = self.get_owner_node();

        while owner.is_some() {
            if owner.as_ref().unwrap().borrow().id.ends_with("_TRASH") {
                return true;
            }
            owner = owner.unwrap().borrow().get_owner_node();
        }

        return false;
    }

    pub fn is_sys_node(&self) -> bool {
        return self.id.starts_with("SYS");
    }
}

impl Meta {
    pub fn from_node(node: NodeRef) -> Option<Meta> {
        let props = node.borrow().get_props();
        let node = node.borrow();
        let children = node.children.borrow();

        if props.doc_type.as_ref()? == &String::from("tuple") {
            let kind = &children[0];

            if kind.borrow().id == "SYS_A13" {
                let tags = children
                    .iter()
                    .skip(1)
                    // .filter(|child| !store.is_sys_node(child))
                    .map(|child| {
                        child
                            .borrow()
                            .props
                            .as_ref()
                            .unwrap()
                            .name
                            .as_ref()
                            .unwrap()
                            .clone()
                    })
                    .collect::<Vec<_>>();

                return Some(Meta::SuperTags(tags));
            }
        }

        return None;
    }
}
