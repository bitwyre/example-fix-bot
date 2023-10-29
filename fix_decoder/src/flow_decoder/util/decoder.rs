#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_doc_comments)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
use serde_json::{Map, Value};
use std::collections::HashMap;

//#[path = "get_tags.rs"] mod get_tags;
use crate::flow_decoder::get_tags::TagName;

#[derive(Debug)]
pub struct JSONMEssage {
    pub tag: String,
    pub value: String,
}

pub fn decode(msg: &[u8], dicts: Vec<TagName>, names: Vec<String>, tags: Vec<u64>) -> String {
    println!("");
    println!(" ====== ");
    println!("enter decoder");

    println!("dicts");
    println!("{:?}", dicts);

    println!("");
    println!(" ====== ");
    println!("Raw bytes to be decoded");
    println!("{:?}", msg);

    let msg_str = String::from_utf8_lossy(&msg);
    let mut results: String = "{".to_string();

    println!("");
    println!(" ====== ");
    println!("strigified message");
    println!("{:?}", msg_str);

    let splits = msg_str.split_terminator("|").collect::<Vec<&str>>();

    println!("");
    println!(" ====== ");
    println!("split message vec");
    println!("{:?}", splits);

    println!("");
    println!(" ====== ");
    println!("Looping over individual tagval");

    for data in &splits {
        let mut name: String = "".to_string();

        println!("");
        println!(" ====== ");
        println!("raw data = {}", data);

        let data = data.split_terminator("=").collect::<Vec<&str>>();
        let keys = data[0];
        let msg = data[1];
        println!("keys     = {:?}", keys);
        println!("msg      = {:?}", msg);

        let keys = keys.parse::<u64>().unwrap();
        println!("keys     = {:?}", keys);

        if tags.contains(&keys) {
            println!("Key exist");
        } else {
            println!("KEY DOES NOT EXIST");
            // do something
        }

        for datum in &dicts {
            if keys == datum.tag {
                name = datum.name.clone();
                let result: String = format!("\"{}\": \"{}\", ", name, msg.to_string());
                results = results + &result;
            }
        }
    }

    results = results + &"}";
    results
}
