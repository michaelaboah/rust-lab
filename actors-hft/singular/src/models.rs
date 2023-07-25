use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::EnumString;
pub type Symbol = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Currency {
    exchange: Exchange,
    symbol: String,
}

#[derive(EnumString, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    #[strum(serialize = "BinanceUsdm", serialize = "binanceusdm")]
    BinanceUsdm = 2,
    #[strum(serialize = "BinanceCoinm", serialize = "binancecoinm")]
    BinanceCoinm = 3,
    #[strum(serialize = "Coinbase", serialize = "coinbase")]
    Coinbase = 6,
    #[strum(serialize = "Kraken", serialize = "kraken")]
    Kraken = 5,
    #[strum(serialize = "Bitfinex", serialize = "bitfinex")]
    Bitfinex = 10,
    #[strum(serialize = "Huobi", serialize = "huobi")]
    Huobi = 7,
    #[strum(
        serialize = "Okx",
        serialize = "okx",
        serialize = "Okex",
        serialize = "okex"
    )]
    #[default]
    Okx = 1,
    #[strum(serialize = "ByBit", serialize = "bybit")]
    ByBit = 9,
    #[strum(serialize = "BitStamp", serialize = "bitstamp")]
    BitStamp = 8,
    #[strum(serialize = "Deribit", serialize = "deribit")]
    Deribit = 4,
}
impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::BinanceUsdm => write!(f, "binanceusdm"),
            Exchange::BinanceCoinm => write!(f, "binancecoinm"),
            Exchange::Coinbase => write!(f, "coinbase"),
            Exchange::Kraken => write!(f, "kraken"),
            Exchange::Bitfinex => write!(f, "bitfinex"),
            Exchange::Huobi => write!(f, "huobi"),
            Exchange::Okx => write!(f, "okx"),
            Exchange::ByBit => write!(f, "bybit"),
            Exchange::BitStamp => write!(f, "bitstamp"),
            Exchange::Deribit => write!(f, "deribit"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instrument {
    exchange: Exchange,
    symbol: String,
    r#type: InstrumentType,
    min_price: f64,
    min_quantity: f64,
    contract_mult: f64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum InstrumentType {
    LinearFuture = 1,
    LinearPerpetual = 2,
    InverseFuture = 3,
    InversePerpetual = 4,
    Spot = 5,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Orderbook {
    pub asks: BTreeMap<ordered_float::OrderedFloat<f64>, f64>,
    pub bids: BTreeMap<ordered_float::OrderedFloat<f64>, f64>,
    pub is_snap: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub enum Side {
    #[default]
    BUY = 1,
    SELL = 2,
}

pub mod normal {
    use std::fmt::Display;

    use super::*;
    use strum::EnumString;
    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct Snapshot {
        pub asks: Vec<(f64, f64)>,
        pub bids: Vec<(f64, f64)>,
        pub symbol: String,
    }

    #[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Stats {
        pub latency_ms: f64,
        pub exchange_status: bool,
    }

    #[derive(
        EnumString, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, Clone, Copy, Default,
    )]
    #[serde(rename_all = "lowercase")]
    pub enum DataTypes {
        #[default]
        #[strum(serialize = "Book", serialize = "book")]
        Book,
        #[strum(serialize = "Trade", serialize = "trade")]
        Trade,
        // #[serde(rename = "snapshot")]
        // #[strum(serialize = "Snapshot", serialize = "snapshot")]
        // BookSnapshot,
    }

    impl Display for DataTypes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DataTypes::Book => write!(f, "book"),
                DataTypes::Trade => write!(f, "trade"),
            }
        }
    }
}
