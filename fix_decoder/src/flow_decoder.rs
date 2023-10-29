#[allow(unused)]
use chrono::prelude::*;
use fefix::prelude::*;
use fefix::tagvalue::{Config, Decoder, RawDecoder};
mod util;
use crate::flow_decoder::util::get_tags;
use anyhow::{anyhow as anyerror, Result as AnyResult, Error};

use serde_json::{Map, Value};

pub const TAG_HEADERS_REQ: &'static [u32] = &[8, 9, 35, 49, 56, 34, 52];
pub const TAG_HEADERS_OPT: &'static [u32] =
    &[115, 128, 90, 91, 50, 142, 57, 143, 116, 144, 129, 145, 43, 97, 122, 212, 213, 347, 369, 627, 628, 629, 640];
pub const TAG_TRAILERS_REQ: &'static [u32] = &[10];
pub const TAG_TRAILERS_OPT: &'static [u32] = &[93, 89];

pub fn flow_decoder(fix_message: &[u8]) -> AnyResult<String> {
    ////println!("input msg:");
    ////println!("{:?}", String::from_utf8_lossy(fix_message));
    let mut headers = Map::new();
    let mut opt_headers = Map::new();
    let mut bodies = Map::new();
    let mut trailers = Map::new();
    let mut final_result = Map::new();

    // let mut decoder = RawDecoder::<Config>::new();
    // decoder.config_mut().set_separator(b'|');
    // let message = decoder.decode(fix_message);

    ////println!("msg object {:?} ", message);

    // let message = message.unwrap();

    // let payload = message.payload().to_string();
    // let bodylength = message.payload().len();
    ////println!("payload:");
    ////println!("{:?}", payload);
    ////println!("\nBodyLength<9>: {:?}", ToString::to_string(&bodylength));

    let msg_str = String::from_utf8_lossy(fix_message);
    let splits = msg_str.split_terminator("|").collect::<Vec<&str>>();
    let last_field = splits.last().ok_or(decode_err())?.split_terminator("=").collect::<Vec<&str>>();
    if last_field.len() < 2 {
        return Err(decode_err());
    }

    let _keys = last_field[0];
    let val = last_field[1];
    ////println!("\nCheckSum<10>: {:?}", val);

    let fix_dictionary = Dictionary::fix44();
    let mut fix_decoder = Decoder::<Config>::new(fix_dictionary.clone());
    fix_decoder.config_mut().set_separator(b'|');
    let msg = fix_decoder.decode(&fix_message)?;
    ////println!("\nnumber of fields: {}", msg.fields().len());

    ////println!("\nraw decoded message");
    ////println!("{:?}", msg);
    ////println!("\nfix_message {:?}", String::from_utf8_lossy(fix_message));

    let fix_dict = Dictionary::fix44();

    ////println!("{:?}\n", fix44::BEGIN_STRING);

    ////println!("looking");

    // parsing body first
    // ordering is not important, thus we freely loop over
    for (tag, value) in msg.fields() {
        let tag = tag.get() as u32;
        let key = fix_dict.field_by_tag(tag).ok_or(decode_err())?.name().to_string();
        let value = value.to_string();

        if TAG_HEADERS_REQ.contains(&tag) {
            ////println!("");
            ////println!("{:?}: {:?}: {:?}", tag, key, value);
            headers.insert(key, Value::String(msg.fv::<&str>(tag)?.to_string()));
        } else if TAG_HEADERS_OPT.contains(&tag) {
            ////println!("");
            ////println!("{:?}: {:?}: {:?}", tag, key, value);
            opt_headers.insert(key, Value::String(msg.fv::<&str>(tag)?.to_string()));
        } else if TAG_TRAILERS_REQ.contains(&tag) {
            continue;
        } else if TAG_TRAILERS_OPT.contains(&tag) {
            ////println!("");
            ////println!("{:?}: {:?}: {:?}", tag, key, value);
            trailers.insert(key, Value::String(msg.fv::<&str>(tag)?.to_string()));
        } else {
            ////println!("");
            ////println!("{:?}: {:?}: {:?}", tag, key, value);
            ////bodies.insert(key, Value::String(msg.fv::<&str>(tag).unwrap().to_string()));
            bodies.insert(key, Value::String(value));
        }
    }

    ////println!("");
    ////println!("result body:");
    ////println!("{:?}", bodies);
    let bodies_str = serde_json::to_string(&bodies.clone())?.as_bytes().to_string();
    let bodylength = serde_json::to_string(&bodies_str)?.as_bytes().len();
    ////println!("{:?}", bodylength);
    ////println!("");
    ////println!("result opt headers:");
    ////println!("{:?}", opt_headers);

    ////println!("");
    ////println!("");
    ////println!("");
    ////println!("");
    ////println!("valnum");
    trailers.insert("CheckSum".to_string(), Value::String(val.to_string()));

    // required headers, ordering is important
    // thus hardcode the orders using serde
    // TODO do something if these required headers are missing
    ////println!("");

    // loop over other headers

    ////println!("{:?}\n", headers);
    ////println!("{:?}\n",  serde_json::to_string(&headers).unwrap());

    final_result.insert("Header".to_string(), Value::Object(headers));
    final_result.insert("Body".to_string(), Value::Object(bodies));
    final_result.insert("Trailer".to_string(), Value::Object(trailers));

    let final_result = serde_json::to_string(&final_result)?;
    ////println!("{}\n", final_result);
    Ok(final_result)
}

fn decode_err() -> Error {
    anyerror!("failed to decode")
}