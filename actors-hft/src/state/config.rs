use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, Hash, PartialEq, Clone, Default)]
pub struct Config {
    #[serde(skip_serializing)]
    pub resources: Resources,
    #[serde(skip_serializing)]
    pub channels: Available,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Clone, Default)]
pub struct Resources {
    pub instrument_url: String,
    pub dispatch_url: String,
    pub dispatch_zmq: String,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Clone, Default)]
pub struct Available {
    pub diff_book: bool,
    pub snap_book: bool,
    pub trade: bool,
    pub candles: bool,
}
