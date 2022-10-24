use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use regex::Regex;

use crate::{block::*, node::*, node_builder::*};

pub struct BlockBuilder {
    store: Rc<RefCell<HashMap<String, BlockRef>>>,
    node_builder: Rc<NodeBuilder>,
}

impl BlockBuilder {
    pub fn new(node_builder: Rc<NodeBuilder>) -> BlockBuilder {
        BlockBuilder {
            store: Rc::new(RefCell::new(HashMap::new())),
            node_builder: Rc::clone(&node_builder),
        }
    }

    pub fn build_blocks(&self) {
        self.node_builder
            .get_nodes()
            .iter()
            .filter(|node| !node.borrow().is_in_trash() && !node.borrow().is_sys_node())
            .for_each(|node| {
                self.build_block(Rc::clone(node));
            })
    }

    pub fn write_pages(&self, output: &String) {
        let output_path = std::path::Path::new(output);

        if !output_path.exists() {
            std::fs::create_dir(output_path).unwrap();
        }

        self.get_blocks()
            .iter()
            .filter(|block| block.borrow().is_page())
            .for_each(|block| {
                let filename = &block.borrow().title;
                let mut filepath = std::path::Path::new(output).to_path_buf().join(filename);
                filepath.set_extension("md");

                let content = block.borrow().get_content(0, true).join("\n");

                std::fs::File::create(filepath)
                    .unwrap()
                    .write(&content.as_bytes())
                    .unwrap();
            })
    }

    pub fn get_blocks(&self) -> Vec<BlockRef> {
        self.store
            .borrow()
            .values()
            .map(|block| Rc::clone(block))
            .collect()
    }

    fn get_block(&self, block_id: &String) -> BlockRef {
        Rc::clone(self.store.borrow().get(block_id).unwrap())
    }

    fn add_block(&self, block: BlockRef) {
        self.store
            .borrow_mut()
            .insert(block.borrow().id.clone(), Rc::clone(&block));
    }

    fn contains_block(&self, block_id: &String) -> bool {
        self.store.borrow().contains_key(block_id)
    }

    fn build_block(&self, node: NodeRef) {
        let id = &node.borrow().id;
        if self.contains_block(id) {
            return;
        }

        let props = node.borrow().get_props();

        let block_ref = Rc::new(RefCell::new(Block {
            id: id.clone(),
            title: String::from(""),
            description: props.description.clone(),
            tags: node.borrow().get_tag_list(),
            metadata: HashMap::new(),
            children: RefCell::new(vec![]),
            doc_type: node.borrow().get_doc_type(),
            ref_count: 0,
        }));

        self.add_block(block_ref);

        self.build_block_title(Rc::clone(&node));
        self.build_block_children(Rc::clone(&node));
    }

    fn build_block_by_id(&self, id: &String) {
        if self.node_builder.contains_node(id) {
            self.build_block(self.node_builder.get_node(id));
        };
    }

    fn build_block_children(&self, node: NodeRef) {
        let id = &node.borrow().id;

        node.borrow()
            .get_children()
            .iter()
            .filter(|child| !child.borrow().is_in_trash() && !child.borrow().is_sys_node())
            .for_each(|child| {
                self.build_block(Rc::clone(child));
                let doc_type = self.get_block(&child.borrow().id).borrow().doc_type.clone();

                match doc_type {
                    DocType::Text => {
                        let child_block = self.get_block(&child.borrow().id);
                        child_block.borrow_mut().ref_count += 1;
                        self.get_block(id)
                            .borrow_mut()
                            .children
                            .borrow_mut()
                            .push(Rc::clone(&child_block));
                    }
                    DocType::Tuple => {
                        let child_block = self.get_block(&child.borrow().id);
                        let children = child_block.borrow().get_children();
                        if children.len() < 2 {
                            return;
                        }
                        let key = children[0].borrow().title.clone();
                        let values: Vec<BlockRef> =
                            children[1..].iter().map(|val| Rc::clone(val)).collect();
                        self.get_block(id).borrow_mut().metadata.insert(key, values);
                    }
                    _ => {}
                }
            });
    }

    fn build_block_title(&self, node: NodeRef) {
        let id = &node.borrow().id;
        let name = node.borrow().get_name();
        if name.is_none() {
            return;
        }

        let name = name.unwrap();
        let re = Regex::new("(.*?)<span data-inlineref-node=(.*?)></span>").unwrap();
        let title = re
            .replace_all(&name, |caps: &regex::Captures| {
                let id = &caps[2][1..&caps[2].len() - 1].to_string();
                self.build_block_by_id(id);
                let child = self.get_block(id);
                format!("{}{}", &caps[1], child.borrow().get_link())
            })
            .to_string();

        self.get_block(id).borrow_mut().title = title;
    }
}
