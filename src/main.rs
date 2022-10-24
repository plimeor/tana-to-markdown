use std::{rc::Rc, time::Instant};

use block_builder::*;
use node_builder::*;

mod block;
mod block_builder;
mod config;
mod node;
mod node_builder;

fn main() {
    let time1 = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let config = config::Config::new(&args).unwrap();

    let mut node_builder = NodeBuilder::new();
    node_builder.load_data_source(config.input);
    node_builder.build_nodes();

    let page_builder = BlockBuilder::new(Rc::new(node_builder));
    page_builder.build_blocks();
    page_builder.write_pages(&config.output);

    let time2 = Instant::now();
    println!("Finish in {:?}", time2.duration_since(time1));

    // let store = store::Store::new(config.input.clone());

    // let calc_end = Instant::now();

    // println!("Calc in: {:?}", calc_end.duration_since(start));

    // let page_store = store.page_store.borrow();
    // page_store.values().for_each(|page| {
    //     page.borrow().write(config.output.clone());
    // });

    // let pages: Vec<page::Page> = store
    //     .get_nodes()
    //     .iter()
    //     .map(|node| page::from_node(&store, node))
    //     .filter(|page| page.is_some())
    //     .map(|page| page.unwrap())
    //     .collect();

    // dbg!(pages.len());

    // pages.iter().for_each(|page| {
    //     // dbg!(&page.tags);
    //     // if page.children.join(" ").contains("合力") {
    //     //     // println!("{:?}", page.children);
    //     // }

    //     page.write(config.output.clone());
    // });
}
