use crate::node::{DocType, Node, NodeRef, Props};
use std::{cell::RefCell, cmp::Ordering, collections::HashMap, fs, io::Read, rc::Rc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DataSource {
    #[serde(rename = "formatVersion")]
    pub format_version: u64,
    pub docs: Vec<OriginNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginNode {
    pub id: String,
    pub props: Option<OriginalProps>,
    #[serde(rename = "touchCounts")]
    pub touch_counts: Option<Vec<u64>>,
    #[serde(rename = "modifiedTs")]
    pub modified_ts: Option<Vec<u64>>,
    pub children: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginalProps {
    pub created: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "_docType")]
    pub doc_type: Option<String>,
    #[serde(rename = "_ownerId")]
    pub owner_id: Option<String>,
    #[serde(rename = "_metaNodeId")]
    pub meta_node_id: Option<String>,
    #[serde(rename = "_sourceId")]
    pub source_id: Option<String>,
}

pub struct NodeBuilder {
    origin_store: HashMap<String, OriginNode>,
    store: Rc<RefCell<HashMap<String, NodeRef>>>,
}

impl NodeBuilder {
    pub fn new() -> NodeBuilder {
        NodeBuilder {
            store: Rc::new(RefCell::new(HashMap::new())),
            origin_store: HashMap::new(),
        }
    }

    pub fn load_data_source(&mut self, filepath: String) {
        let mut input_file = fs::OpenOptions::new().read(true).open(filepath).unwrap();
        let mut input_content = String::new();
        input_file.read_to_string(&mut input_content).unwrap();
        let data_source: DataSource = serde_json::from_str(&input_content.clone()).unwrap();

        data_source.docs.into_iter().for_each(|origin_node| {
            self.origin_store
                .insert(origin_node.id.clone(), origin_node);
        });
    }

    pub fn build_nodes(&self) {
        self.origin_store
            .values()
            .for_each(|origin_node| self.build_node(origin_node))
    }

    pub fn get_nodes(&self) -> Vec<NodeRef> {
        self.store
            .borrow()
            .values()
            .map(|node| Rc::clone(node))
            .collect()
    }

    pub fn get_node(&self, id: &String) -> NodeRef {
        Rc::clone(self.store.borrow().get(id).unwrap())
    }

    fn add_node(&self, node: NodeRef) {
        self.store
            .borrow_mut()
            .insert(node.borrow().id.clone(), Rc::clone(&node));
    }

    pub fn contains_node(&self, id: &String) -> bool {
        self.store.borrow().contains_key(id)
    }

    pub fn build_node(&self, origin_node: &OriginNode) {
        let node_id = origin_node.id.clone();
        if self.contains_node(&node_id) {
            return;
        }

        let node_ref: NodeRef = Rc::new(RefCell::new(Node::new(
            node_id.clone(),
            None,
            RefCell::new(vec![]),
        )));

        self.add_node(node_ref);

        self.build_node_props(&origin_node);

        self.build_node_child(&origin_node);
    }

    pub fn build_node_by_id(&self, id: &String) {
        if self.origin_store.contains_key(id) {
            let origin_node = self.origin_store.get(id).unwrap();
            self.build_node(origin_node);
        }
    }

    fn build_node_child(&self, origin_node: &OriginNode) -> Option<bool> {
        let node_id = &origin_node.id;
        origin_node
            .children
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .for_each(|child_id| {
                let origin_child_node = self.origin_store.get(child_id);
                if origin_child_node.is_none() {
                    return;
                }

                self.build_node(origin_child_node.unwrap());

                let child_node = self.get_node(child_id);

                self.get_node(node_id)
                    .borrow_mut()
                    .children
                    .borrow_mut()
                    .push(child_node);
            });

        self.get_node(node_id)
            .borrow()
            .children
            .borrow_mut()
            .sort_by(|a, b| {
                let mut a_is_tuple = false;
                let mut b_is_tuple = false;

                if let DocType::Tuple = a.borrow().get_doc_type() {
                    a_is_tuple = true;
                }

                if let DocType::Tuple = b.borrow().get_doc_type() {
                    b_is_tuple = true;
                }

                match (a_is_tuple, b_is_tuple) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (_, _) => Ordering::Equal,
                }
            });

        Some(true)
    }

    fn build_node_props(&self, origin_node: &OriginNode) -> Option<bool> {
        let node_id = &origin_node.id;
        let origin_props = origin_node.props.clone()?;

        let mut props = Props {
            created: origin_props.created,
            name: origin_props.name,
            description: origin_props.description,
            doc_type: origin_props.doc_type,
            owner_node: None,
            meta_node: None,
            source_node: None,
        };

        self.get_node(node_id).borrow_mut().props = Some(props.clone());

        if origin_props.owner_id.is_some() {
            let owner_id = &origin_props.owner_id?;
            self.build_node_by_id(owner_id);
            if self.contains_node(owner_id) {
                props.owner_node = Some(self.get_node(owner_id));
            }
        }

        if origin_props.meta_node_id.is_some() {
            let meta_id = &origin_props.meta_node_id?;
            self.build_node_by_id(meta_id);
            if self.contains_node(meta_id) {
                props.meta_node = Some(self.get_node(meta_id));
            }
        }

        if origin_props.source_id.is_some() {
            let source_id = &origin_props.source_id?;
            self.build_node_by_id(source_id);
            if self.contains_node(source_id) {
                props.source_node = Some(self.get_node(source_id));
            }
        }

        self.get_node(node_id).borrow_mut().props = Some(props.clone());

        Some(true)
    }
}
