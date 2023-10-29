use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub begin_string: String,
    pub body_length: u16,
    pub msg_type: String,
    pub sender_comp_id: String,
    pub target_comp_id: String,
    pub msg_seq_num: u16,
    pub sending_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trailer {
    pub check_sum: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogonMsg {
    pub header: Header,
    pub body: LogonBody,
    pub trailer: Trailer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogonBody {
    pub encrypt_method: u8,
    pub heart_bt_int: u8,
    pub raw_data: String,
    pub password: String,
    //pub cancel_orders_on_disconnect: String,
}
