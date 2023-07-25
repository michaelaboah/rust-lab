use std::sync::Arc;

use serde::{Deserialize, Serialize};
use singular::models::{normal, Exchange};

use super::client::{Extra, StreamRequest};

/// Possible events a this server can respond with
///
/// All variants will serialize into their lowercase forms
///
/// ### Variants
/// - __Hb__: (Heartbeat variant) - Testing if client is still connected, else disconnect client. _See_: [`WsClient`]
/// - __Error__: (Error variant) - Any/All errors that should be sent back to client.
/// - __Data__: (Data variant) - Clients requested data. Usually a `String`
/// - __Info__: (Information variant) - Server has information for the user
///

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "event")]
pub enum ServerResponse {
    Hb,
    Data {
        payload: serde_json::Value,
        meta: Option<Meta>,
        stats: Option<normal::Stats>,
    },
    Info {
        message: String,
    },
    /// Used for maintenice
    Warn {
        message: String,
    },
    Error {
        message: String,
    },
    Subscribed {
        channel: String,
    },
    Unsubscribed {
        channel: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub channel: String,
    pub exchange: Exchange,
    pub data_type: normal::DataTypes,
    pub asset_class: String,
    pub symbol: String,
    // pub options: Option<Extra>,
}

impl From<StreamRequest> for Meta {
    fn from(value: StreamRequest) -> Self {
        Self {
            channel: value.to_string(),
            exchange: value.exchange,
            data_type: value.data_type,
            asset_class: value.asset_class,
            symbol: value.symbol,
            // options: value.options,
        }
    }
}
/// This clones the !copy fields
impl From<Arc<StreamRequest>> for Meta {
    fn from(value: Arc<StreamRequest>) -> Self {
        Self {
            channel: value.to_string(),
            exchange: value.exchange,
            data_type: value.data_type.clone(),
            asset_class: value.asset_class.clone(),
            symbol: value.symbol.clone(),
            // options: value.options.clone(),
        }
    }
}
