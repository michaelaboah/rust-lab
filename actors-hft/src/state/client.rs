use super::server::ServerResponse;
use crate::routes::symbols::retrieve_symbols;
// use crate::{api::authentication::Credentials, routes::retrieve_symbols, CONFIG};
use serde::{Deserialize, Serialize};
use singular::models::{normal::DataTypes, Exchange};
use std::{collections::HashMap, fmt::Display, time::Instant};

#[derive(Debug)]
pub struct WsState {
    pub client_id: usize,
    hb: Instant,
    is_auth: bool,
    pub messages: HashMap<StreamRequest, RequestState>,
    // pub cache: HashMap<Arc<StreamRequest>, Vec<gq::Types>>,
    max_writes: usize,
}

impl WsState {
    pub fn new(id: usize) -> Self {
        Self {
            client_id: id,
            hb: Instant::now(),
            is_auth: false,
            messages: HashMap::new(),
            max_writes: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "event")]
/// Possible events a client can invoke
///
/// All variants will serialize into their lowercase forms
///
/// ### Variants
/// - Subscribe: Client is requesting data that fufills the string field in [`ClientRequest`]
/// - Unsubscribe: Client is unrequesting data that fufills the string field in [`ClientRequest`]
/// - Auth: Client is attempting to authenticate. REQUIRED BEFORE USING API
///
/// ## Example
/// ```
///  let str = r#"{"event":"auth", "key": "Some kind of key"}"#;
///
///  let parsed: ClientEvent = serde_json::from_str(str).unwrap();
///
///  println!("Data: {:?}", serde_json::to_string(&parsed));
/// ```
pub enum ClientEvent {
    Subscribe(ClientRequest),
    Unsubscribe(ClientRequest),
    ///
    /// `{"event": "auth", "key": "afdasdaf"}`
    /// `{"event": "auth", "creds": { "username": "user", "password": "pass" }}`
    Auth {
        key: Option<String>,
        // creds: Option<Credentials>,
    },
    Status,
}

/// Expected structure of Client's json request
///
/// ### Example(s):
/// ```json
///
///     {"event":"subscribe", "channel":"binance.spot.ticker.btcusdt"}
///     {"event":"subscribe", "channels":["binance.spot.book.btcusdt",
///     "kraken.spot.trade.DOGE-USD"]}
/// ```
///
///
/// ### Fields
/// - `event`: Client's specified event. See [`ClientEvent`]
/// - `channel`: Client's specified channel containing information for: `{exchange}.{asset}.{type}.{symbol}`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientRequest {
    pub channel: Option<String>,
    // pub channels: Option<Vec<String>>,
    #[cfg(feature = "interval")]
    pub options: Option<Extra>,
}

fn str_to_request(channel_str: &str) -> Result<StreamRequest, ServerResponse> {
    use std::str::FromStr;
    let mut stream_request = StreamRequest::default();
    let broken: Vec<&str> = channel_str.split(".").collect::<_>();

    match Exchange::from_str(broken[0]) {
        Ok(ex) => {
            stream_request.exchange = ex;
        }
        Err(_) => return Err(ServerResponse::Error {
            message:
                "Incorrect exchange name. Only lowercase and first letter capitalization supported"
                    .into(),
        }),
    };

    match DataTypes::from_str(broken[2]) {
        Ok(rename) => stream_request.data_type = rename,
        Err(_) => {
            return Err(ServerResponse::Error {
                message: "Incorrect DataType provided. Only Trade (trade), Book (book) supported."
                    .into(),
            })
        }
    };

    futures::executor::block_on(validate_symbols(
        broken[0].to_string(),
        broken[3].to_string(),
    ))?;

    stream_request.symbol = broken[3].to_string();

    Ok(stream_request)
}

async fn validate_symbols(exc_str: String, sym: String) -> Result<(), ServerResponse> {
    // let map = retrieve_symbols(None, None).await.unwrap();
    // if let Some(symbols) = map.get(&exc_str.to_lowercase()) {
    //     let search = symbols
    //         .as_array()
    //         .expect("The symbols json has changed from a Map<String, Vec<String>>")
    //         .iter()
    //         .find(|s| match s.as_str() {
    //             Some(t) => sym == t,
    //             None => false,
    //         });
    //
    //     if search.is_none() {
    //         return Err(ServerResponse::Error {
    //             message: format!(
    //                 "The provided symbol: {sym} is not provided in the exchange, {exc_str}"
    //             ),
    //         });
    //     }
    // }
    Ok(())
}

impl ClientRequest {
    pub fn to_request(&self) -> Result<StreamRequest, ServerResponse> {
        if let Some(c) = self.channel.as_ref() {
            return str_to_request(c);
        }

        Err(ServerResponse::Error {
            message: format!("No request provided"),
        })
    }

    // pub fn to_requests(&self) -> Option<Vec<Result<StreamRequest, ServerResponse>>> {
    //     if let Some(strs) = self.channels.as_ref() {
    //         return Some(strs.iter().map(|r| str_to_request(r)).collect());
    //     }
    //     None
    // }
}

/// The request sent to the Capture/Arbitration layer to recieve data from ZMQ
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct StreamRequest {
    pub exchange: Exchange,
    pub data_type: DataTypes,
    pub symbol: String,
    pub asset_class: String,
    pub options: Option<Extra>,
}

impl Display for StreamRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let exc_str = self.exchange.to_string();
        let pipe_str = self.data_type.to_string();
        write!(f, "{exc_str}.spot.{pipe_str}.{}", self.symbol)
    }
}

#[derive(Debug, Default, Clone)]
pub enum RequestState {
    Book {
        recieve_snapshot: bool,
    },
    #[default]
    Trade,
}

/// Provides extended functionality for data formatting and requests
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Extra {
    /// Duration of time before data aggregation
    pub interval: Option<usize>,
    /// Number of data points before aggregation
    pub tick: Option<usize>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    #[cfg(feature = "interval")]
    #[test]
    fn test_payload() {
        let mut req = ClientRequest {
            channel: Some("binance.spot.book.DOGE-USD".into()),
            channels: None,
            options: None,
        };

        let stream_req = StreamRequest {
            symbol: "DOGE-USD".into(),
            ..StreamRequest::default()
        };

        assert_eq!(stream_req, req.to_request().unwrap());

        req.channel = None;
        req.channels = Some(vec![
            "binance.spot.book.DOGE-USD".into(),
            "kraken.spot.trade.DOGE-USD".into(),
            "okx.spot.trade.DOGE-USD".into(),
        ]);

        let stream_reqs = req.to_requests().unwrap();

        assert_eq!(
            &StreamRequest {
                symbol: "DOGE-USD".into(),
                ..StreamRequest::default()
            },
            stream_reqs[0].as_ref().unwrap()
        );
        assert_eq!(
            &StreamRequest {
                exchange: (Exchanges::Kraken, Config::default()),
                data_type: DataTypes::Trade,
                symbol: "DOGE-USD".into(),
                asset_class: "spot".into(),
                options: None,
            },
            stream_reqs[1].as_ref().unwrap()
        );
        assert_eq!(
            &StreamRequest {
                exchange: (Exchanges::Okx, Config::default()),
                data_type: DataTypes::Trade,
                symbol: "DOGE-USD".into(),
                asset_class: "spot".into(),
                options: None,
            },
            stream_reqs[2].as_ref().unwrap()
        );
    }

    #[tokio::test]
    async fn test_symbol_validation() {
        let map = retrieve_symbols(None, None).await.unwrap();

        let exchange = map.get("huobi").unwrap();
        // dbg!(&exchange);
        let search = exchange
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s.as_str().unwrap() == "btcusdt");
        dbg!(&search);
    }

    #[test]
    fn interval_syntax() {
        let ex_json = json!({"interval": 1}).to_string();
        let converted = serde_json::from_str::<Extra>(&ex_json).is_ok();
        dbg!(&converted);
        assert!(converted);
    }
    #[test]
    fn tick_syntax() {
        let ex_json = json!({"tick": 1}).to_string();
        let converted = serde_json::from_str::<Extra>(&ex_json).is_ok();
        dbg!(&converted);
        assert!(converted);
    }
    #[test]
    fn client_extras() {
        let ex_json = json!({"event": "subscribe", "channel": "okx.spot.trade.BTC-USDT", "options": json!({"interval": 1})}).to_string();

        let converted = serde_json::from_str::<ClientRequest>(&ex_json).is_ok();
        dbg!(&converted);
        assert!(converted);
    }
}
