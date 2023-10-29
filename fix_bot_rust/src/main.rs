use chrono::Utc;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use bitwyre_logger::*;
use ringbuf::HeapRb;
use fix_decoder::flow_decoder;
use fix_encoder::flow_encoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::Duration;
use serde_json;
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::prelude::*;
use rand::prelude::*;

struct OrderBookUpdate {
    pub price: String,
    pub qty: String,
    pub side: i8,
}

const BASE_ASSET: f64 = 100000000.0;
const QUOTE_ASSET: f64 = 100000000.0;
const MAX_ORDER: usize = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    info!("starting to bind");

    let addr_orderentry = env::var("FIX_ORDERENTRY_ADDRESS").unwrap();
    let addr_marketdata = env::var("FIX_MARKETDATA_ADDRESS").unwrap();
    let addr_dropcopy = env::var("FIX_DROPCOPY_ADDRESS").unwrap();

    let username =  env::var("USERNAME").unwrap();
    let password =  env::var("PASSWORD").unwrap();
    let symbol = env::var("SYMBOL").unwrap();

    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (cancel_tx, _cancel_rx) = tokio::sync::mpsc::channel::<()>(5);
    let cancel_tx_sigint = cancel_tx.clone();
    let cancel_tx_sigterm = cancel_tx.clone();

    let map_order_ids = Arc::new(Mutex::new(HashMap::new()));
    let map_order_ids_clone = map_order_ids.clone();

    let map_order_return = Arc::new(Mutex::new(HashMap::new()));
    let map_order_return_clone = map_order_return.clone();
    let map_order_return_clone_2 = map_order_return.clone();
    let map_order_return_clone_3 = map_order_return.clone();

    let (mut orderbook_prod, mut orderbook_cons) = HeapRb::<OrderBookUpdate>::new(1024).split();

    let task_sigint = tokio::spawn(async move {
        let sigint = tokio::signal::ctrl_c();
        sigint.await.expect("Failed to receive SIGINT");
        let _ = cancel_tx_sigint.send(()).await;
    });

    let task_sigterm = tokio::spawn(async move {
        let sigterm_stream = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        tokio::pin!(sigterm_stream);
        while let Some(_) = sigterm_stream.recv().await {
            let _ = cancel_tx_sigterm.send(()).await;
        }
    });
    

    // === INITIAL ORDER ====
    let mut buf = vec![0; 1024];
    info!("connect to FIX orderentry: {:?}", addr_orderentry);
    let mut stream = TcpStream::connect(addr_orderentry.clone()).await.unwrap();

    let mut json_logon = json!({
        "Header": {
            "BeginString": "FIX.4.4",
            "MsgType": "A",
            "MsgSeqNum": "1",
            "SenderCompID": "SENDER",
            "TargetCompID": "BITWYRE",
            "SendingTime": "20160802-21:14:38.717"
        },
        "Body": {
            "Username": username.clone(),
            "Password": password.clone(),
        },
        "Trailer": {}
    });
    json_logon["Header"]["SendingTime"] = Value::String(get_timestamp());
    let result_json = serde_json::to_string(&json_logon).unwrap_or_default();
    let fix_logon = flow_encoder::flow_encoder(&result_json);
    info!("FIX orderentry logon msg = {:?}", fix_logon.clone());

    let fix_logon = fix_logon.as_bytes().to_owned();
    let _ = stream.write(&fix_logon).await;
    let b = stream.read(&mut buf).await;

    let buf = &buf[0..b.unwrap()];
    
    info!("\nreturn msg = {:?}", String::from_utf8_lossy(buf));

    // buy
    for n in 0..2 {
        info!("buy order {}", n);
        let ts = Utc::now().timestamp().to_string();
        let ran = &ts[ts.len() - 3..];
        let price = "24".to_owned()+ran;
        let qty = "0.0002".to_owned()+ran;
        let orderid = create_order(&price, &qty, Some(1)).await;

        if !orderid.is_empty() {
            map_order_return.lock().await.insert(orderid, true);
        }

       std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // sell
    for n in 0..2 {
        info!("sell order {}", n);
        let ts = Utc::now().timestamp().to_string();
        let ran = &ts[ts.len() - 3..];
        let price = "25".to_owned()+ran;
        let qty = "0.0001".to_owned()+ran;
        let orderid = create_order(&price, &qty, Some(2)).await;

        if !orderid.is_empty() {
            map_order_return.lock().await.insert(orderid, true);
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    // === END OF INITIAL ORDER ====


    let get_execution_report = tokio::spawn(async move {
        info!("connect to FIX dropcopy: {:?}", addr_dropcopy);
        let mut stream = TcpStream::connect(addr_dropcopy).await.unwrap();
        
        let mut json_logon = json!({
            "Header": {
                "BeginString": "FIX.4.4",
                "MsgType": "A",
                "MsgSeqNum": "1",
                "SenderCompID": "SENDER",
                "TargetCompID": "BITWYRE",
                "SendingTime": "20160802-21:14:38.717"
            },
            "Body": {
                "Username": username.clone(),
                "Password": password.clone(),
            },
            "Trailer": {}
        });

        let result_json = serde_json::to_string(&json_logon).unwrap_or_default();
        let fix_logon = flow_encoder::flow_encoder(&result_json);

        info!("dropcopy logon = {:?}", fix_logon.clone());

        let fix_logon = fix_logon.as_bytes().to_owned();
        let _ = stream.write(&fix_logon).await;
    
        loop {
            if map_order_ids_clone.clone().lock().await.len() >= map_order_return_clone_2.clone().lock().await.len() {
                info!("orders ready to cancel");
                break;
            }
            
            let mut buf_2 = vec![0; 1024];
            let c = stream.read(&mut buf_2).await.unwrap();
            let buf_r = &buf_2.clone()[0..c];
            if buf_r.len() == 0 {
                return;
            }

            let msg_str = String::from_utf8_lossy(buf_r);
            let result: Result<String, _> = flow_decoder::flow_decoder(buf_r);
             
            if let Err(_e) = result {
                error!("error format msg = {}", msg_str);
                continue;
            }

            let json_str = result.unwrap();
            
            if let Some(json_msg) = serde_json::from_str::<Value>(json_str.as_str()).ok() {
                let order_id = json_msg["Body"]["ClOrdID"].as_str().unwrap().to_owned();
                json_logon["Header"]["SendingTime"] = Value::String(get_timestamp());

                info!("");
                info!("execution report order_id = {}", order_id);

                let mut map_order_ids = map_order_ids_clone.lock().await;
                let map_return = map_order_return_clone_2.lock().await;
 
                if map_return.contains_key(&order_id) {
                    info!("execution report ready to cancel = {:?}", String::from_utf8_lossy(buf_r));
                    map_order_ids.insert(order_id, true);
                }

                info!("total ordered = {}", map_return.len());   
                info!("ready to cancel = {}", map_order_ids.len());   
            }
        }

        _ = stream.flush().await;
        _ = stream.shutdown().await;
    });

    tokio::spawn(async move {
        info!("connect to FIX marketdata: {:?}", addr_marketdata);
        let mut stream = TcpStream::connect(addr_marketdata).await.unwrap();

        //market data request
        let msg = "8=FIX.4.4|9=120|35=V|34=2|49=BITWYREUSER|52=20200120-11:51:40.000|56=Bitwyre|262=2|263=1|264=1|265=0|146=1|55=btc_usdt_spot|267=1|269=0|10=052|";
        info!("market data logon = {:?}", msg);

        let msg = msg.to_string().as_bytes().to_owned();
        let _ = stream.write(&msg).await;
    
        loop {
            if map_order_return_clone_3.clone().lock().await.len() >= MAX_ORDER {
                break;
            }

            let mut buf_2 = vec![0; 1024];
            let c = stream.read(&mut buf_2).await.unwrap();
            let buf_r = &buf_2.clone()[0..c];
            buf_2.clear();

            if buf_r.len() == 0 {
                return ;
            }

            info!("");
            info!("market data response = {:?}", String::from_utf8_lossy(buf_r));

            let result: Result<String, _> = flow_decoder::flow_decoder(buf_r);         
            if let Err(_e) = result {
                continue;
            }

            let json_str = result.unwrap();

            if let Some(json_msg) = serde_json::from_str::<Value>(json_str.as_str()).ok() {
                if json_msg["Body"]["Symbol"].as_str().unwrap().to_owned() == symbol.clone() {
                    if !orderbook_prod.is_full() {
                        let ob_update = OrderBookUpdate {
                            price: json_msg["Body"]["MDEntryPx"].as_str().unwrap_or_default().to_owned(),
                            qty: json_msg["Body"]["MDEntryPx"].as_str().unwrap_or_default().to_owned(),
                            side: json_msg["Body"]["Side"].as_str().unwrap_or_default().to_owned().parse().unwrap(),
                        };

                        if let Err(_e) = orderbook_prod.push(ob_update) {
                            error!("failed send order book");
                        }
                    }
                }
            }
        }

        _ = stream.flush().await;
        _ = stream.shutdown().await;

        info!("finish market data. order reached the limit");
    });

    tokio::spawn(async move {
        loop {
            if let Some(ob) = orderbook_cons.pop() {
                let mut price_str = ob.price;

                if price_str.contains("-") {
                    price_str = price_str.replace("-", "");
                }

                let mut rng = StdRng::from_entropy();

                let price: f64 = price_str.parse().unwrap();
                let mut price_in_base_asset = (price / BASE_ASSET) as i64;
                price_in_base_asset = price_in_base_asset - rng.gen_range(10..500);
                
                
                let qty: f64 = ob.qty.parse().unwrap();
                let mut qty_in_quote_asset = (qty / QUOTE_ASSET) / QUOTE_ASSET;

                qty_in_quote_asset = qty_in_quote_asset + rng.gen_range(0.001..0.009);
                // let qty_in_quote_asset = rng.gen_range(0.0001..0.002);
                    
                let orderid = create_order(&price_in_base_asset.to_string(), &qty_in_quote_asset.to_string(), Some(ob.side)).await;
                if !orderid.is_empty() {
                    let mut map_order = map_order_return_clone.lock().await;
                    map_order.insert(orderid, true);
                }
            }
        }
    });


    tokio::select! {
        _ = task_sigint => {
            info!("Task for listening Cancel completed.");
        }
        _ = task_sigterm => {
            info!("Task for SIGTERM completed.");
        }
        _ = get_execution_report => {
            info!("Task for getting execution report completed.");
        }

        _ = shutdown_rx => {
            info!("Cancelling tasks...");
            let _ = cancel_tx.send(()).await;
            let _ = cancel_tx.send(()).await;
            let _ = cancel_tx.send(()).await;
        }
    }

    for key in map_order_ids.lock().await.keys() {
       let _ = create_cancel_order(key.clone()).await;
       std::thread::sleep(std::time::Duration::from_secs(2));
    }

    info!("Finished");
    std::process::exit(0);

    Ok(())
}

fn randomized_order(price_str: &str, qty_str: &str, side_order: Option<i8>) -> String {
    let user_uuid = env::var("USER_UUID").unwrap();
    let symbol = env::var("SYMBOL").unwrap();

    let mut json_new = json!({
        "Header": {
            "BeginString": "FIX.4.4",
            "MsgType": "D",
            "MsgSeqNum": "1",
            "SenderCompID": "SENDER",
            "TargetCompID": "BITWYRE",
            "SendingTime": "20160802-21:14:38.717"
        },
        "Body": {
            "Account":"36113dc7-9760-485b-976b-e37a85224a94",
            "OrderQty": "7000",
            "OrdType": "2",
            "Side": "1",
            "Price":"15000",
            "Symbol": "btc_usdt_spot",
            "TimeInForce":"1",
            "ExpireTime":"20231012-16:16:50.000000000",
            "TransactTime": "20180920-18:14:19.492"
        },
        "Trailer": {}
    });

    let mut rng = thread_rng();
    let mut price: f64 = rng.gen_range(10000.0..35000.0);
    let mut qty: f64 = rng.gen_range(0.0005..1.0);
    let mut side: i8 = rng.gen_range(1..=2);
    
    if !price_str.is_empty() {
        info!("price_str = {}", price_str);
        price = price_str.parse().unwrap();
    }

    if !qty_str.is_empty() {
        qty = qty_str.parse().unwrap();
    }

    if let Some(s) = side_order {
        side = s;
    }
   
    json_new["Body"]["Account"] = Value::String(user_uuid);
    json_new["Body"]["Price"] = Value::String(price.to_string());
    json_new["Body"]["OrderQty"] = Value::String(qty.to_string());
    json_new["Body"]["Side"] = Value::String(side.to_string());
    json_new["Body"]["Symbol"] = Value::String(symbol);
    let timestamp_str = get_timestamp();
    json_new["Header"]["SendingTime"] = Value::String(timestamp_str.clone());
    json_new["Header"]["TransactTime"] = Value::String(timestamp_str);
    let result_json = serde_json::to_string(&json_new).unwrap_or_default();
    let encoded_fix = flow_encoder::flow_encoder(&result_json);

    encoded_fix
}

fn cancel_order_request(order_id: String) -> String {
    let user_uuid = env::var("USER_UUID").unwrap();
    let symbol = env::var("SYMBOL").unwrap();

    let mut json_new = json!({
        "Header": {
            "BeginString": "FIX.4.4",
            "MsgType": "F",
            "MsgSeqNum": "1",
            "SenderCompID": "SENDER",
            "TargetCompID": "BITWYRE",
            "SendingTime": "20160802-21:14:38.717"
        },
        "Body": {
            "OrigClOrdID":"36113dc7-9760-485b-976b-e37a85224a94",
            "OrderQty": "0.1",
            "Symbol": "btc_usdt_spot",
            "Account":"36113dc7-9760-485b-976b-e37a85224a94",
            "TransactTime": "20180920-18:14:19.492"
        },
        "Trailer": {}
    });

    json_new["Body"]["Account"] = Value::String(user_uuid);
    json_new["Body"]["Symbol"] = Value::String(symbol);
    json_new["Body"]["OrigClOrdID"] = Value::String(order_id.clone());
    let timestamp_str = get_timestamp();
    json_new["Header"]["SendingTime"] = Value::String(timestamp_str.clone());
    json_new["Header"]["TransactTime"] = Value::String(timestamp_str);
    let result_json = serde_json::to_string(&json_new).unwrap_or_default();
    let encoded_fix = flow_encoder::flow_encoder(&result_json);

    encoded_fix
}

async fn create_order(price: &str, qty: &str, side: Option<i8>) -> String {
    let addr_orderentry = env::var("FIX_ORDERENTRY_ADDRESS").unwrap();

    let mut stream = TcpStream::connect(addr_orderentry.clone()).await.unwrap();
    info!("");
    info!("");
    let mut buf_2 = vec![0; 1024];
    let msg_2 = randomized_order(price, qty, side);
    info!("sending new order = {:?}", msg_2);

    let msg_2 = msg_2.as_bytes().to_owned();
    let _ = stream.write(&msg_2).await;
    let b = stream.read(&mut buf_2).await;

    let buf_2 = &buf_2[0..b.unwrap()];
    info!("");
    info!("return order msg = {:?}", String::from_utf8_lossy(buf_2));

    let json_str = flow_decoder::flow_decoder(buf_2);
    if let Some(json_msg) = serde_json::from_str::<Value>(json_str.unwrap().as_str()).ok() {
        return json_msg["Body"]["OrderID"].as_str().unwrap_or_default().to_owned();
    }

    "".to_owned()
}

async fn create_cancel_order(order_id: String) {
    let addr_orderentry = env::var("FIX_ORDERENTRY_ADDRESS").unwrap();

    let mut stream = TcpStream::connect(addr_orderentry.clone()).await.unwrap();
    info!("");
    info!("");
    let mut buf_2 = vec![0; 1024];
    let msg_2 = cancel_order_request(order_id);
    info!("sending new cancel order = {:?}", msg_2);

    let msg_2 = msg_2.as_bytes().to_owned();
    let _ = stream.write(&msg_2).await;
    let b = stream.read(&mut buf_2).await;

    let buf_2 = &buf_2[0..b.unwrap()];
    info!("");
    info!("return cancel msg = {:?}", String::from_utf8_lossy(buf_2));
}

fn get_timestamp() -> String {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let d = UNIX_EPOCH + Duration::from_secs(t.as_secs());
    let datetime = DateTime::<Local>::from(d);
    datetime.format("%Y%m%d-%H:%M:%S.%f").to_string()
}