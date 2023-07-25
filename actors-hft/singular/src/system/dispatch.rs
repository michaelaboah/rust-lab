use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Duration,
};

use super::orderbook::OrderbookManagementSystem;
use crate::{event::EventType, models::*};

#[derive(Debug)]
pub enum DispatchCommands {
    Subscribe {
        event_type: EventType,
        symbol: String,
    },
    Unsubscribe {
        event_type: EventType,
        symbol: String,
    },
}

/// Spawns and manages adpaters
/// holds state regarding clients
///
///
/// Handling state
/// Multiple adapters
/// |--->
///
/// Adapters, Symbol, Clients
///
/// Adapter:
///     -> Symbols
///         -> Clients
/// HashMap<Adapters, BTreeMap<Symbols, BTreeSet<Clients>>>
#[derive(Debug)]
pub struct DispatchSystem {
    // subs_tx: Sender<DispatchCommands>,
    orderbook_system: OrderbookManagementSystem,
    state: HashMap<Exchange, BTreeMap<String, BTreeSet<()>>>,
    adpater_handlers: Vec<DemoHandler>,
}

impl DispatchSystem {
    pub fn new() -> Self {
        Self {
            orderbook_system: OrderbookManagementSystem::new(),
            state: HashMap::new(),
            adpater_handlers: Vec::new(),
        }
    }

    pub async fn build_adapters(&mut self) -> &mut Self {
        let okx_handle = Demo::new("wss://ws.okx.com:8443/ws/v5/public")
            .await
            .run()
            .await;

        self.adpater_handlers.push(okx_handle);
        self
    }

    pub fn get_adapter(&self, index: usize) -> Option<&DemoHandler> {
        self.adpater_handlers.get(index)
    }
}
use crate::adapters::less::*;
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn help() {
    let mut dispatch = DispatchSystem::new();

    dispatch.build_adapters().await;

    let handler = dispatch.get_adapter(0).unwrap();

    handler
        .dispatch_sender
        .send(DemoCmd::Sub {
            symbol: "BTC-USDT".into(),
            event_kind: EventType::Trade,
        })
        .unwrap();

    let rx = handler.dispatch_receiver.clone();

    // let orderbook = OrderbookManagementSystem::new();
    // let orderbook_tx = orderbook.start();

    std::thread::spawn(move || {
        while let Ok(v) = rx.clone().recv() {
            dbg!(v);
            // orderbook_tx.send("Hello Orderbook system".into()).unwrap();
        }
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

    ()
}
