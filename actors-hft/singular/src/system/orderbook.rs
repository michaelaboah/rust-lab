use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::collections::HashMap;

use crate::models::{normal::Snapshot, Orderbook, Side, Symbol};
use crossbeam::channel;
#[derive(Debug)]
pub struct OrderbookManagementSystem {
    orderbook_map: HashMap<Symbol, HashMap<usize, Box<Orderbook>>>,
}

impl OrderbookManagementSystem {
    pub fn new() -> Self {
        Self {
            orderbook_map: HashMap::new(),
        }
    }

    pub fn start(&self) -> channel::Sender<String> {
        let (tx, rx) = channel::bounded::<String>(100);

        std::thread::spawn(move || loop {
            let iter = rx.clone().into_iter();
            let par_iter = iter.par_bridge();

            par_iter.for_each(|s| {
                dbg!(s);
            });
        });

        tx
    }

    fn register_orderbook(&mut self, symbol: Symbol, backup_index: usize) {
        println!(
            "OrderbookManagementSystem: registering orderbook for {} at backup_id {}",
            symbol, backup_index
        );

        self.orderbook_map
            .entry(symbol)
            .or_insert_with(HashMap::new)
            .insert(backup_index, Box::new(Orderbook::default()));
    }

    fn deregister_orderbook(&mut self, symbol: Symbol) {
        println!(
            "OrderbookManagementSystem: deregistering orderbook for {}",
            symbol
        );
        self.orderbook_map.remove(&symbol);
    }

    pub fn update_level(&self, orderbook: &mut Orderbook, side: Side, price: f64, quantity: f64) {
        match side {
            Side::BUY if quantity > 0.0 => {
                *orderbook.bids.get_mut(&price.into()).unwrap() = quantity;
            }
            Side::BUY => {
                orderbook.bids.remove(&price.into());
            }
            Side::SELL if quantity > 0.0 => {
                *orderbook.asks.get_mut(&price.into()).unwrap() = quantity;
            }
            Side::SELL => {
                orderbook.asks.remove(&price.into());
            }
        }
    }

    fn update_orderbook(
        &mut self,
        symbol: Symbol,
        backup_index: usize,
        side: Side,
        is_snapshot: bool,
        price: f64,
        quantity: f64,
    ) {
        if let Some(backup_map) = self.orderbook_map.get_mut(&symbol) {
            if let Some(orderbook) = backup_map.get_mut(&backup_index) {
                if (orderbook.is_snap && !is_snapshot) || (!orderbook.is_snap && is_snapshot) {
                    // Update snapshot status
                    orderbook.is_snap = is_snapshot;
                    // Clear bids and asks
                    orderbook.bids.clear();
                    orderbook.asks.clear();
                }

                match side {
                    Side::BUY if quantity > 0.0 => {
                        *orderbook.bids.get_mut(&price.into()).unwrap() = quantity;
                    }
                    Side::BUY => {
                        orderbook.bids.remove(&price.into());
                    }
                    Side::SELL if quantity > 0.0 => {
                        *orderbook.asks.get_mut(&price.into()).unwrap() = quantity;
                    }
                    Side::SELL => {
                        orderbook.asks.remove(&price.into());
                    }
                }
            }
        } else {
            // TODO: getting orderbook updates for unknown instruments
        }
    }

    fn snapshot(&self, symbol: Symbol, backup_index: usize) -> Option<Snapshot> {
        if let Some(backup_map) = self.orderbook_map.get(&symbol) {
            let mut message = Snapshot::default();

            for (price, quantity) in &backup_map[&backup_index].bids {
                println!("Bids: {:?}", (price, quantity));
                message.bids.push((**price, *quantity));
            }

            for (price, quantity) in &backup_map[&backup_index].asks {
                println!("Asks: {:?}", (price, quantity));
                message.asks.push((**price, *quantity));
            }

            return Some(message);
        }
        None
        // TODO: handle request for snapshot for unknown orderbook
    }

    fn disconnect(&mut self, symbol: Symbol, backup_id: usize) {
        println!("OrderbookManagementSystem: clearing invalid orderbooks");
        if let Some(backup_map) = self.orderbook_map.get_mut(&symbol) {
            backup_map.get_mut(&backup_id).map(|orderbook| {
                orderbook.bids.clear();
                orderbook.asks.clear();
            });
        }
    }
}
