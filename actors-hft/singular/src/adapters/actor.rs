use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::{mpsc, oneshot};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::{
    event::{self, Event, EventType},
    models::normal::DataTypes,
    transmute::{self, okx::TradeRaw},
};
type SocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct MyActor {
    receiver: mpsc::Receiver<ActorMessage>,
    write: SplitSink<SocketStream, Message>,
    read: SplitStream<SocketStream>,
    subscriptions: HashMap<SocketRequest, Vec<mpsc::Sender<String>>>,
}

enum ActorMessage {
    Sub {
        request: SocketRequest,
        respond_to: mpsc::Sender<String>,
    },
    Unsub {
        request: SocketRequest,
        respond_to: mpsc::Sender<SocketRequest>,
    },
}

impl MyActor {
    pub async fn new(receiver: mpsc::Receiver<ActorMessage>, ws_url: &str) -> Self {
        let (write_stream, _) = connect_async(Url::parse(ws_url).expect("url"))
            .await
            .unwrap();

        let (write, read) = write_stream.split();
        Self {
            read,
            write,
            receiver,
            subscriptions: HashMap::new(),
        }
    }
    async fn handle_message(&mut self, msg: ActorMessage) {
        let _ = match msg {
            ActorMessage::Sub {
                request,
                respond_to,
            } => self.subscribe(request, respond_to).await,
            ActorMessage::Unsub {
                request,
                respond_to,
            } => self.unsubscribe(request, respond_to).await,
        };
    }

    async fn subscribe(
        &mut self,
        request: SocketRequest,
        client_tx: mpsc::Sender<String>,
    ) -> Result<(), ()> {
        self.subscriptions
            .entry(request.clone())
            .or_default()
            .push(client_tx);
        let message = match request.data_type {
            DataTypes::Trade => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "trades",
                    "instId": request.symbol
                }]
            })),
            DataTypes::Book => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "books",
                    "instId": request.symbol
                }]
            })),
            // DataTypes::OrderbookSnapshot => Some(serde_json::json!({
            //     "op": "subscribe",
            //     "args": [{
            //         "channel": "books5",
            //         "instId": "BTC-USDT"
            //     }]
            // })),
        };
        return if let Some(m) = message {
            self.write.send(Message::Text(m.to_string())).await.unwrap();
            Ok(())
        } else {
            todo!();
            // Err(DemoError {
            //     message: format!(
            //         "The provided Event Type is not available on this exchange: {}",
            //         kind
            //     ),
            // })
        };
    }

    async fn unsubscribe(
        &mut self,
        request: SocketRequest,
        client_tx: mpsc::Sender<SocketRequest>,
    ) -> Result<(), ()> {
        let message = match request.data_type {
            DataTypes::Trade => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "trades",
                    "instId": request.symbol
                }]
            })),
            DataTypes::Book => Some(serde_json::json!({
                "op": "subscribe",
                "args": [{
                    "channel": "books",
                    "instId":  request.symbol
                }]
            })),
            // EventType::OrderbookSnapshot => Some(serde_json::json!({
            //     "op": "unsubscribe",
            //     "args": [{
            //         "channel": "books5",
            //         "instId": symbol
            //     }]
            // })),
        };
        return if let Some(m) = message {
            self.write.send(Message::Text(m.to_string())).await.unwrap();
            Ok(())
        } else {
            todo!();
            // Err(DemoError {
            //     message: format!(
            //         "The provided Event Type is not available on this exchange: {}",
            //         kind
            //     ),
            // })
        };
    }

    pub fn parse(raw_str: &str) -> serde_json::Result<event::Event> {
        let de_ser: transmute::okx::OkxRaw<TradeRaw> = serde_json::from_str(raw_str)?;
        Ok(event::Event::Trade(de_ser.into()))
    }
}

async fn run_my_actor(mut actor: MyActor) {
    loop {
        tokio::select! {
           Some(Ok(val)) = actor.read.next() => {
                if let Ok(parse) = MyActor::parse(&val.to_string()) {
                    let subs = match parse {
                        Event::Trade(t) => {
                            dbg!(&t);
                            let request = SocketRequest { symbol: t.symbol, data_type: DataTypes::Trade};
                            &actor.subscriptions.get(&request).unwrap()[..]
                        }
                        Event::OrderbookUpdate(t) => todo!(),
                        Event::OrderbookSnapshot(t) => todo!(),
                    };
                    send_to_clients(subs, &val.to_string()).await;
                }
           }
           Some(msg) = actor.receiver.recv() => {
               actor.handle_message(msg).await;
           }
        }
    }
}

async fn send_to_clients(clients: &[mpsc::Sender<String>], value: &str) {
    for tx in clients.iter() {
        tx.send(value.to_string()).await.unwrap();
    }
}
#[derive(Clone)]
pub struct MyActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl MyActorHandle {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = MyActor::new(receiver, "wss://ws.okx.com:8443/ws/v5/public").await;
        tokio::spawn(run_my_actor(actor));

        Self { sender }
    }
}

#[tokio::test]
async fn actor_test() {
    let handle = MyActorHandle::new().await;

    let (send, mut recv) = mpsc::channel(400);
    let request = SocketRequest {
        symbol: "BTC-USDT".into(),
        data_type: DataTypes::Trade,
    };

    let _ = handle
        .sender
        .send(ActorMessage::Sub {
            respond_to: send.clone(),
            request,
        })
        .await;

    let request = SocketRequest {
        symbol: "ETH-USDT".into(),
        data_type: DataTypes::Trade,
    };

    let _ = handle
        .sender
        .send(ActorMessage::Sub {
            respond_to: send,
            request,
        })
        .await;

    while let Some(t) = recv.recv().await {
        // println!("Received: {}", t);
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Default)]
pub struct SocketRequest {
    symbol: String,
    data_type: DataTypes,
}
