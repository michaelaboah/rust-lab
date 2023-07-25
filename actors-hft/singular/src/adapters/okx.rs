use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};

use crate::{
    event::{Event, Trade},
    interfaces::Adapter,
    models::Symbol,
};
use async_trait::async_trait;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt as _, StreamExt as _,
};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;
type SocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct Okx {
    connected: bool,
    write: Option<SplitSink<SocketStream, Message>>,
    /// By default this HeapAllocated Ring buffer will have a capacity of `1024`
    /// This buffer will be used for data only
    data_buffer: Arc<Mutex<VecDeque<(String, String)>>>,
    /// This buffer will only be for non-data messages Eg: Hb, status, infor & warn messages
    message_buffer: Arc<Mutex<VecDeque<String>>>,
    subscriptions: BTreeMap<String, watch::Receiver<String>>,
    senders: Arc<Mutex<BTreeMap<String, watch::Sender<String>>>>,
}

struct Connected;
struct Unconnected;

impl Okx {
    /// Connect to provided exchange and start reading the stream into buffer
    pub async fn connect(mut self, url: &str) -> Self {
        let (ws_stream, _) = connect_async(Url::parse(url).unwrap())
            .await
            .expect("Failed to connect");

        let (write, mut read) = ws_stream.split();
        self.write = Some(write);
        self.connected = true;
        let write_to_data_buffer = self.data_buffer.clone();
        // let write_to_message_buffer = self.message_buffer.clone();
        let _ = tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                let mut data_buff = write_to_data_buffer.lock().unwrap();
                // let string = msg.to_string();
                data_buff.push_back(("BTC-USDT".into(), msg.to_string()));
                // if string.contains("tradeId") {
                //     data_buff.push_front(msg.to_string());
                // } else if string.contains("orderbookId") {
                //     data_buff.push_front(msg.to_string());
                // } else {
                //     let mut mess_buff = write_to_message_buffer.lock().unwrap();
                //     mess_buff.push_front(string);
                // }
            }
        });

        let buff: Arc<Mutex<VecDeque<(String, String)>>> = self.data_buffer.clone();
        let sends: Arc<Mutex<BTreeMap<String, watch::Sender<String>>>> = self.senders.clone();

        let _ = tokio::spawn(async move {
            while !buff.lock().unwrap().is_empty() {
                println!("Hello");
                // for (sym, tx) in sends.lock().unwrap().iter() {
                //     let mut read_buff = buff.lock().unwrap();
                //     if !read_buff.is_empty() {
                //         tx.send(read_buff.pop_back().unwrap().1).unwrap();
                //     }
                // }
            }
        });
        self
    }
}

// #[async_trait(?Send)]
impl Okx {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            write: None,
            connected: true,
            data_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(4000))),
            message_buffer: Arc::new(Mutex::new(VecDeque::new())),
            subscriptions: BTreeMap::new(),
            senders: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    async fn send_message(&mut self, message: String) {
        self.write
            .as_mut()
            .expect("Writer is not None")
            .send(Message::Text(message))
            .await
            .unwrap();
    }
    pub async fn subscribe_orderbook(&mut self, symbol: Symbol) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "books",
                "instId": symbol.to_string()

            }]
        });

        self.send_message(message.to_string()).await;
        let (tx, rx) = watch::channel("Hello".to_string());
        self.subscriptions.insert(symbol.clone(), rx);
        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.clone(), tx);

        // dbg!(&self.subscriptions, &txs);
    }

    pub async fn subscribe_trade(&mut self, symbol: Symbol) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "trades",
                "instId": symbol
            }]
        });

        self.send_message(message.to_string()).await;
        let (tx, rx) = watch::channel("Hello".to_string());
        self.subscriptions.insert(symbol.clone(), rx);

        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.clone(), tx);
        println!("Complete");
    }

    pub async fn subscribe_orderbook_snapshot(&mut self, symbol: crate::models::Symbol) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "books5",
                "instId": symbol
            }]
        });

        self.send_message(message.to_string()).await;

        let (tx, rx) = watch::channel("Hello".to_string());

        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.clone(), tx);

        self.subscriptions.insert(symbol.clone(), rx);
    }
    pub async fn unsubscribe_orderbook(&mut self, symbol: &str) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "books",
                "instId": symbol
            }]
        });

        self.send_message(message.to_string()).await;
        let (tx, rx) = watch::channel("Hello".to_string());
        self.subscriptions.insert(symbol.to_string(), rx);
        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.to_string(), tx);
    }

    pub async fn unsubscribe_trade(&mut self, symbol: &str) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "trades",
                "instId": symbol
            }]
        });

        self.send_message(message.to_string()).await;
        let (tx, rx) = watch::channel("Hello".to_string());
        self.subscriptions.insert(symbol.to_string(), rx);

        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.to_string(), tx);
        println!("Complete");
    }

    pub async fn unsubscribe_orderbook_snapshot(&mut self, symbol: &str) {
        let message = serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "books5",
                "instId": symbol
            }]
        });

        self.send_message(message.to_string()).await;

        let (tx, rx) = watch::channel("Hello".to_string());

        let sends = self.senders.clone();
        let mut txs = sends.lock().unwrap();
        txs.insert(symbol.to_string(), tx);

        self.subscriptions.insert(symbol.to_string(), rx);
    }
    pub fn get_receiver(&self, symbol: &str) -> Option<&watch::Receiver<String>> {
        self.subscriptions.get(symbol)
    }

    pub async fn reconnect(&mut self) -> Result<(), ()> {
        if self.connected {
            return Ok(());
        }
        let (ws_stream, _) =
            connect_async(Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap())
                .await
                .expect("Failed to connect");

        let (write, read) = ws_stream.split();

        self.write = Some(write);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const URL: &'static str = "wss://ws.okx.com:8443/ws/v5/public";
    #[tokio::test]
    async fn test_connect() {
        // let client = awc::Client::default();
        let mut okx_adapter = Okx::new().connect(URL).await;
        okx_adapter.subscribe_trade("BTC-USDT".into()).await;
    }

    #[tokio::test]
    async fn test_buffer() {
        let mut okx_adapter = Okx::new().connect(URL).await;
        okx_adapter.subscribe_trade("BTC-USDT".into()).await;
        let buff_read = okx_adapter.data_buffer.clone();
        // let que = buff_read.lock().unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }

    #[tokio::test]
    async fn test_read() {
        let mut okx_adapter = Okx::new().connect(URL).await;
        okx_adapter.subscribe_orderbook("BTC-USDT".into()).await;
        // okx_adapter.stream().await;

        // okx_adapter.subscribe_trade("BTC-USDT".into()).await;
        let read = okx_adapter.get_receiver("BTC-USDT").unwrap();
        let mut read = read.clone();
        while read.changed().await.is_ok() {
            println!("Recieved: {}", *read.borrow());
        }
    }
}
