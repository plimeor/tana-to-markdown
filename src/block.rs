use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::node::*;

pub type BlockRef = Rc<RefCell<Block>>;

pub struct Block {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, Vec<BlockRef>>,
    pub children: RefCell<Vec<BlockRef>>,
    pub doc_type: DocType,
    pub ref_count: usize,
}

impl Block {
    pub fn get_link(&self) -> String {
        format!("[[{}]]", self.title)
    }

    pub fn get_children(&self) -> Vec<BlockRef> {
        self.children
            .borrow()
            .iter()
            .map(|child| Rc::clone(child))
            .collect()
    }

    pub fn is_page(&self) -> bool {
        let is_field = self.tags.contains(&String::from("field-definition"));
        let is_supertag = self.tags.contains(&String::from("supertag"));
        let title_less = self.title.len() == 0;
        if is_field || is_supertag || title_less {
            return false;
        }

        let has_tag = self.tags.iter().filter(|tag| tag != &"todo").count() > 0;
        let has_field = self.metadata.len() > 0;

        match self.doc_type {
            DocType::Text => has_tag || has_field,
            _ => false,
        }
    }

    pub fn get_content(&self, level: usize, extend: bool) -> Vec<String> {
        let mut content = vec![];
        let is_page = self.is_page();
        let prefix = "  ".repeat(level);
        let next_level = if is_page { level } else { level + 1 };

        if is_page && !extend {
            content.push(format!("{}- {}", prefix, self.get_link()));
            return content;
        }

        if is_page {
            content.push(format!("{}title:: {}", prefix, &self.title));
        }

        if self.tags.len() > 0 {
            let tags = self
                .tags
                .iter()
                .map(|tag| format!("#{}", &tag))
                .collect::<Vec<String>>()
                .join(" ");

            if is_page {
                content.push(format!("{}tags:: {}", prefix, tags));
            } else {
                content.push(format!("{}- {} {}", prefix, tags, &self.title))
            }
        } else if !is_page {
            content.push(format!("{}- {}", prefix, &self.title))
        }

        if self.metadata.len() > 0 || self.description.is_some() {
            let prefix = if is_page {
                prefix
            } else {
                "  ".repeat(next_level)
            };
            content.push(format!("{}- Metadata", prefix));
            if self.description.is_some() {
                content.push(format!(
                    "{}  - Description: {}",
                    prefix,
                    self.description.as_ref().unwrap()
                ))
            }

            self.metadata.iter().for_each(|(key, values)| {
                content.push(format!("{}  - {}", prefix, key));
                let mut sub_contents: Vec<String> = values
                    .iter()
                    .map(|block| block.borrow().get_content(next_level, false))
                    .flatten()
                    .map(|str| format!("    {}", str))
                    .collect();
                content.append(&mut sub_contents);
            })
        }

        let children = self.get_children();
        let mut child_contents: Vec<String> = children
            .iter()
            .map(|block| block.borrow().get_content(next_level, false))
            .flatten()
            .map(|str| format!("{}", str))
            .collect();

        content.append(&mut child_contents);
        content
    }
}
