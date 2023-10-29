#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_doc_comments)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
use serde_json::{Deserializer, Map, Value};
use std::collections::HashMap;
use std::fs;

//#[path = "get_tags.rs"] mod get_tags;
use crate::flow_decoder::get_tags::TagName;

#[derive(Debug)]
struct JSON {
    key: String,
    msg: String,
}

pub fn encode(msg: String, dicts: Vec<TagName>, names: Vec<String>, tags: Vec<u64>) -> String {
    println!("");
    println!(" ====== ");
    println!("enter encoder");

    println!("dicts");
    println!("{:?}", dicts);

    println!("");
    println!(" ====== ");
    println!("Raw message to be encoded");
    //println!("{:?}", msg);

    //let json: serde_json::Value = serde_json::from_str(&msg).expect("JSON was not well-formatted");

    let path = "./json_fix_example.json";
    let data = fs::read_to_string(path).expect("Unable to read file");
    let mut rawjson: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    let mut results: String = "".to_string();

    println!("");
    println!(" ====== ");
    println!("Deserialized message to be encoded");
    println!("{:?}", rawjson);

    for (mut key, mut value) in rawjson.as_object().unwrap() {
        let value = value.as_str().unwrap().to_string();
        println!("");
        println!(" ====== ");
        println!("{:?}", key);
        println!("{:?}", value);

        if names.contains(&key) {
            println!("Key exist");
        } else {
            println!("KEY DOES NOT EXIST");
            // do something
        }

        for datum in &dicts {
            if key.to_string() == datum.name {
                let tag = datum.tag.clone();
                println!("tag {:?}", tag);

                //let result: JSONMEssage = JSONMEssage{tag: name, value: msg.to_string()};
                //let result: String = format!("{{{}: {} }}", name, msg.to_string());
                //results.append(& mut vec![result])
                //let result: String = format!("\"{}\": \"{}\", ", name, msg.to_string());
                //println!("{}", result);
                //results = results + &result;
                let result: String = format!("{}={}|", tag, value);
                println!("{}", result);
                results = results + &result;
            }
        }
    }

    results
}
