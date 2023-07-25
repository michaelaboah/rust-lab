use std::fmt::Display;

use crate::models::{Exchange, Side};
use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize)]
// #[derive(Clone, Debug)]
// pub struct Event {
//     pub r#type: EventType,
//     pub buffer: [u8; 8192],
// }
//
// impl Event {
//     pub fn buffer(&mut self) -> &mut [u8; 8192] {
//         &mut self.buffer
//     }
// }

#[derive(Clone, Debug)]
pub enum Event {
    Trade(Trade),
    OrderbookUpdate(OrderbookUpdate),
    OrderbookSnapshot(OrderbookSnapshot),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum EventType {
    Trade = 1,
    OrderbookUpdate = 2,
    AdapterDisconnect = 3,
    OrderbookSnapshot = 4,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Trade => write!(f, "Trade"),
            EventType::OrderbookUpdate => write!(f, "OrderbookUpdate"),
            EventType::AdapterDisconnect => todo!(),
            EventType::OrderbookSnapshot => write!(f, "OrderbookSnapshot"),
        }
    }
}

// #[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct OrderbookSnapshot {
    exchange: Exchange,
    symbol: String,
    // payload: [u8; 20000],
}

// #[derive(Serialize, Deserialize)]
#[derive(Default, Clone, Debug)]
pub struct OrderbookUpdate {
    exchange: Exchange,
    symbol: String,
    side: Side,
    price: f64,
    quantity: f64,
    is_snapshot: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Trade {
    pub exchange: Exchange,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
}
