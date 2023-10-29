use fefix::json::FieldOrGroup;
use fefix::prelude::*;

use std::num::NonZeroU32;

pub fn flow_encoder(json_fix_message: &str) -> String {
    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" ====== input json_fix_message ====== ");
    //println!("{}", json_fix_message);

    const orig_msg: &str = include_str!("./json_fix_example.json");

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" ====== asserting json_fix_message ====== ");
    //assert!(orig_msg == json_fix_message, "a = {:?}, ========================= b = {:?}", orig_msg, json_fix_message);
    //println!("assert passed");

    let dictionary = fefix::Dictionary::fix44();
    let mut decoder = <fefix::json::Decoder>::new(dictionary.clone());
    let mut encoder = <fefix::tagvalue::Encoder>::new();
    encoder.config_mut().set_separator(b'|');
    let mut buffer = Vec::new();

    let json_msg = decoder.decode(json_fix_message.as_bytes()).unwrap();
    let msg_type = json_msg.fv(fix44::MSG_TYPE).unwrap();
    let begin_string: &[u8] = json_msg.fv(fix44::BEGIN_STRING).unwrap();

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" ====== json_fix_message ====== ");
    //println!("{:?}", json_fix_message);

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!("====== json_msg ====== ");
    //println!("{:?}", json_msg);

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!("====== begin_string ====== ");
    //println!("{:?}", begin_string);
    //let begin_string: &[u8] = b"abcefg";
    //println!("{:?}", begin_string);
    //println!("{:?}", String::from_utf8_lossy(begin_string));

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" json_msg ====== ");
    ////println!("{:?}", json_msg.field_map(fix44::MSG_TYPE));

    let mut fix_msg_builder = encoder.start_message(begin_string, &mut buffer, msg_type);

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" ====== iterating over header fields ====== ");
    //println!("");
    for (field_name, field_value) in json_msg.iter_header_fields() {
        let field = dictionary.field_by_name(field_name).expect("Invalid FIX.4.2 field!");

        match field_value {
            FieldOrGroup::Field(s) => {
                //println!("{:?}, {:?}, {:?}", field.tag(), field_name, s);
                if fefix::FieldType::to_string(&field.tag()) != "8".to_string() {
                    // begin string has already been handled in start message - beginstring
                    fix_msg_builder.set(field.tag(), s.as_ref());
                }
            }
            FieldOrGroup::Group(_g) => {}
        }
    }

    for (field_name, field_value) in json_msg.iter_trailer_fields() {
        let field = dictionary.field_by_name(field_name).expect("Invalid FIX.4.2 field!");

        match field_value {
            FieldOrGroup::Field(s) => {
                //println!("{:?}, {:?}, {:?}", field.tag(), field_name, s);
                fix_msg_builder.set(field.tag(), s.as_ref());
            }
            FieldOrGroup::Group(_g) => {}
        }
    }

    //println!("");
    //println!(" ====== iterating over body fields ====== ");
    //println!("");
    for (field_name, field_value) in json_msg.iter_body_fields() {
        let field = dictionary.field_by_name(field_name).expect("Invalid FIX.4.2 field!");

        match field_value {
            FieldOrGroup::Field(s) => {
                fix_msg_builder.set(field.tag(), s.as_ref());
            }
            FieldOrGroup::Group(g) => {
                for v in g.iter() {
                    for (f_name, f_v) in v {
                        if let FieldOrGroup::Field(ss) = f_v {
                            let field = dictionary.field_by_name(f_name).expect("Invalid FIX.4.2 field!");
                            fix_msg_builder.set(field.tag(), ss.as_ref());
                        }
                    }
                }
            }
        }
    }

    //println!("");
    //println!(" ====== iterating over trailing fields ====== ");
    //println!("");

    //println!("");
    //println!(" ====== done iterating over fields ====== ");
    //println!(" ====== ");
    //println!("");
    //println!("");

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" fix_msg_builder ====== ");
    //println!("{:?}", fix_msg_builder);

    let fix_msg = fix_msg_builder.done().0;

    //println!("");
    //println!("");
    //println!(" ====== ");
    //println!(" fix_msg ====== ");
    //println!("{:?}", fix_msg);

    //println!("Successful conversion from JSON syntax to tag=value|.");
    //println!();
    //println!("{}", String::from_utf8_lossy(fix_msg));

    let hardmsg: &[u8] = &[56, 61, 70, 73, 88, 46, 52, 46, 52, 124];
    //println!("{}", String::from_utf8_lossy(hardmsg));

    fix_msg.to_string()
}
