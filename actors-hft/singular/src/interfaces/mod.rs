use crate::{
    event::{Event, EventType, Trade},
    models::{Exchange, Symbol},
};
use async_trait::async_trait;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt as _, StreamExt as _,
};
use std::{
    collections::VecDeque,
    net::TcpStream,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
pub type SocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
#[async_trait(?Send)]
pub trait Adapter {
    /// Create an adapter with a provided exchange. Also generates a connection to the implemented
    /// client;
    async fn new(exchange: Exchange) -> Self
    where
        Self: Sized;

    async fn subscribe_orderbook(&mut self, symbol: Symbol);
    async fn subscribe_trade(&mut self, symbol: Symbol);
    async fn subscribe_orderbook_snapshot(&mut self, symbol: Symbol);

    // fn parse<T: Deserialize>(&self, buffer: &str) -> serde_json::Value {
    //     serde_json::from_str(buffer).unwrap()
    // }

    /// Parse the buffer into a valid value in ['Event'] and push it onto the adapter's buffer
    async fn parse(&mut self);

    // fn run(&self);
    fn connected(&self) -> bool;
    fn buffer_lock(&mut self) -> &mut Arc<Mutex<VecDeque<String>>>;

    async fn reconnect(&mut self) -> Result<(), ()>;
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     struct MockAdapter {
//         buffer: VecDeque<Event>,
//     }
//
//     #[async_trait(?Send)]
//     impl Adapter for MockAdapter {
//         async fn new(exchange: Exchange) -> Self {
//             MockAdapter {
//                 buffer: VecDeque::new(),
//             }
//         }
//
//         async fn subscribe_orderbook(&mut self, symbol: Symbol) {}
//
//         async fn subscribe_trade(&mut self, symbol: Symbol) {}
//
//         async fn subscribe_orderbook_snapshot(&mut self, symbol: Symbol) {}
//
//         fn parse(&self, buffer: &str) {}
//
//         fn connected(&self) -> bool {
//             true
//         }
//
//         async fn reconnect(&mut self) -> Result<(), ()> {
//             Ok(())
//         }
//         fn buffer(&mut self) -> &mut VecDeque<Event> {
//             &mut self.buffer
//         }
//     }
//
//     #[test]
//     fn test_send_trade() {
//         // Create a mock adapter
//         let mut adapter = MockAdapter {
//             buffer: VecDeque::new(),
//         };
//
//         // Create a sample event and call send_trade
//         // let event = Trade::default();
//         adapter.send(Event::Trade(Trade::default()));
//
//         // Assert that the event was pushed into the buffer
//         assert_eq!(adapter.buffer().len(), 1);
//
//         dbg!(adapter.buffer);
//     }
// }
