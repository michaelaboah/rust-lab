use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::{
    event::{self, EventType},
    models::normal::DataTypes,
    transmute::{self, okx::TradeRaw},
};
type SocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Variants of state that we send to the dispatch system
#[derive(Debug)]
pub enum DispatchEvents {
    Data(String),
    DataReal(event::Event),
    Info(String),
    Error(DemoError),
}

#[derive(Debug)]
pub struct Demo {
    write: SplitSink<SocketStream, Message>,
    read: SplitStream<SocketStream>,
    // state:
}

impl Demo {
    pub async fn new(ws_url: &str) -> Self {
        let (write_stream, _) = connect_async(Url::parse(ws_url).expect("url"))
            .await
            .unwrap();

        let (write, read) = write_stream.split();
        Self { read, write }
    }

    pub fn parse(raw_str: &str) -> serde_json::Result<event::Event> {
        let de_ser: transmute::okx::OkxRaw<TradeRaw> = serde_json::from_str(raw_str)?;
        Ok(event::Event::Trade(de_ser.into()))
    }
}

#[derive(Debug)]
pub struct DemoHandler {
    pub dispatch_sender: UnboundedSender<DemoCmd>,
    pub dispatch_receiver: crossbeam::channel::Receiver<DispatchEvents>,
}
#[derive(Debug)]
pub struct DemoError {
    pub message: String,
}
use crossbeam::channel;
impl Demo {
    pub async fn run(mut self) -> DemoHandler {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let (sync_tx, sync_rx) = channel::bounded(10);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(Ok(val)) = self.read.next() => {
                        if let Ok(t) = Self::parse(val.to_string().as_str()) {
                        sync_tx.send(DispatchEvents::DataReal(t)).unwrap();
                        }
                        sync_tx.send(DispatchEvents::Data(val.to_string())).unwrap();
                    }
                    Some( cmd) = rx.recv() => {
                        let res = match cmd {
                           DemoCmd::Sub { ref symbol, event_kind } => self.subscribe(event_kind, symbol).await,
                           DemoCmd::Unsub { ref symbol, event_kind } => self.unsubscribe(event_kind, symbol).await,
                           DemoCmd::Reconnect {ref url}=> {
                                self.reconnect(url).await;
                                Ok(())
                           }
                        };

                        if let Err(e) = res {
                            sync_tx.send(DispatchEvents::Error(e)).unwrap();
                        }
                    }
                }
            }
        });

        DemoHandler {
            dispatch_sender: tx,
            dispatch_receiver: sync_rx,
        }
    }

    // impl Demo<Running> {
    async fn subscribe(&mut self, kind: EventType, symbol: &str) -> Result<(), DemoError> {
        let message = match kind {
            EventType::Trade => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "trades",
                    "instId": symbol
                }]
            })),
            EventType::OrderbookUpdate => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "books",
                    "instId": "BTC-USDT"
                }]
            })),
            EventType::AdapterDisconnect => None,
            EventType::OrderbookSnapshot => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "books5",
                    "instId": "BTC-USDT"
                }]
            })),
        };
        return if let Some(m) = message {
            self.write.send(Message::Text(m.to_string())).await.unwrap();
            Ok(())
        } else {
            Err(DemoError {
                message: format!(
                    "The provided Event Type is not available on this exchange: {}",
                    kind
                ),
            })
        };
    }

    async fn unsubscribe(&mut self, kind: EventType, symbol: &str) -> Result<(), DemoError> {
        let message = match kind {
            EventType::Trade => Some(serde_json::json!({
                "op": "unsubscribe",
                "args": [{
                    "channel": "trades",
                    "instId": symbol
                }]
            })),
            EventType::OrderbookUpdate => Some(serde_json::json!({
                "op": "unsubscribe",
                "args": [{
                    "channel": "books",
                    "instId": "BTC-USDT"
                }]
            })),
            EventType::AdapterDisconnect => None,
            EventType::OrderbookSnapshot => Some(serde_json::json!({
                "op": "unsubscribe",
                "args": [{
                    "channel": "books5",
                    "instId": "BTC-USDT"
                }]
            })),
        };
        return if let Some(m) = message {
            self.write.send(Message::Text(m.to_string())).await.unwrap();
            Ok(())
        } else {
            Err(DemoError {
                message: format!(
                    "The provided Event Type is not available on this exchange: {}",
                    kind
                ),
            })
        };
    }
    async fn reconnect(&mut self, ws_url: &str) {
        let (write_stream, _) = connect_async(Url::parse(ws_url).expect("url"))
            .await
            .unwrap();

        let (write, read) = write_stream.split();
        self.write = write;
        self.read = read;
    }
    // pub appendj
}
#[derive(Debug)]
pub enum DemoCmd {
    Sub {
        symbol: String,
        event_kind: EventType,
    },
    Unsub {
        symbol: String,
        event_kind: EventType,
    },
    Reconnect {
        url: String,
    },
}
#[cfg(test)]
mod test {
    use tokio::time;

    use super::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn example() {
        let DemoHandler {
            dispatch_sender: tx,
            dispatch_receiver,
        } = Demo::new("wss://ws.okx.com:8443/ws/v5/public")
            .await
            .run()
            .await;

        tx.send(DemoCmd::Sub {
            symbol: "BTC-USDT".into(),
            event_kind: EventType::Trade,
        })
        .unwrap();

        tokio::spawn(async move {
            while let Ok(t) = dispatch_receiver.recv() {
                if let DispatchEvents::Data(r) = t {
                    println!("{}", r);
                }
            }
        });

        time::sleep(time::Duration::from_secs(2)).await;

        tx.send(DemoCmd::Unsub {
            symbol: "BTC-USDT".into(),
            event_kind: EventType::Trade,
        })
        .unwrap();
    }
}
