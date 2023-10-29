#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_doc_comments)]
#![allow(unreachable_code)]
#![allow(unused_mut)]

use serde::de::value::StringDeserializer;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct TagName {
    pub tag: u64,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Name {
    name: String,
}

//#[derive(Debug, Deserialize, Serialize)]
pub fn get_tags() -> (Vec<TagName>, Vec<String>, Vec<u64>) {
    println!("");
    println!("");
    println!(" ====== ");
    println!("enter get tags");
    let path = "./tags.json";
    let data = fs::read_to_string(path).expect("Unable to read file");
    let mut rawdict: Vec<Value> = serde_json::from_str(&data).expect("Unable to parse");

    println!("tags {:?}", rawdict);

    println!("");
    println!(" ====== ");

    let mut dict: Vec<TagName> = vec![];
    let mut tags: Vec<u64> = vec![];
    let mut names: Vec<String> = vec![];

    for elem in rawdict.iter_mut() {
        let tag: u64 = elem["tag"].as_i64().unwrap() as u64;
        let name: String = elem["name"].as_str().unwrap().to_string();

        tags.append(&mut vec![tag.clone()]);
        names.append(&mut vec![name.clone()]);

        let tagname: TagName = TagName { tag, name };
        dict.append(&mut vec![tagname]);
    }

    (dict, names, tags)
}
