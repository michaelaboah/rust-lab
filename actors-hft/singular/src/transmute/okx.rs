use crate::event;
use crate::models::{Exchange, Side};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::mem;

const RAW_BOOK: &'static str = r#"
        {
            "arg": {
                "channel": "books",
                "instId": "BTC-USDT"
            },
            "action": "update",
            "data": [
                {
                    "asks": [
                        ["30557.3", "0", "0", "0"],
                        ["30557.6", "0.51065898", "0", "1"]
                    ],
                    "bids": [
                        ["30545", "0.51069492", "0", "2"],
                        ["30544.9", "0.17474", "0", "1"]
                    ],
                    "ts": "1688060541909",
                    "checksum": -1316686072,
                    "seqId": 12815993309,
                    "prevSeqId": 12815993303
                }
            ]
        }
    "#;

#[derive(Serialize, Deserialize)]
pub struct OkxRaw<Data> {
    arg: Arg,
    action: Option<String>,
    data: Vec<Data>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arg {
    channel: String,
    inst_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BookUpdateRaw {
    asks: Vec<[String; 4]>,
    bids: Vec<[String; 4]>,
    ts: String,
    checksum: i64,
    seq_id: u64,
    prev_seq_id: u64,
}

const RAW_TRADE: &'static str = r#"
        {
            "arg": {
                "channel": "trades",
                "instId": "BTC-USDT"
            },
            "data": [
                {
                    "instId": "BTC-USDT",
                    "tradeId": "426790906",
                    "px": "30460.1",
                    "sz": "0.0010244",
                    "side": "sell",
                    "ts": "1688085963425"
                }
            ]
        }"#;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeRaw {
    inst_id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    trade_id: u128,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    px: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    sz: f64,
    side: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    ts: u128,
}

impl From<OkxRaw<TradeRaw>> for event::Trade {
    fn from(mut value: OkxRaw<TradeRaw>) -> Self {
        Self {
            symbol: mem::take(&mut value.data[0].inst_id),
            exchange: Exchange::Okx,
            side: if value.data[0].side == "sell" {
                Side::SELL
            } else {
                Side::BUY
            },
            price: value.data[0].px,
            quantity: value.data[0].sz,
        }
    }
}
#[test]
fn test_book_transform() {
    let valid = serde_json::from_str::<OkxRaw<BookUpdateRaw>>(RAW_BOOK).is_ok();
    assert!(valid);
    let valid = serde_json::from_str::<OkxRaw<TradeRaw>>(RAW_TRADE);
    let trade: event::Trade = valid.unwrap().into();

    dbg!(trade);
    // assert!(&valid.is_ok());
}
