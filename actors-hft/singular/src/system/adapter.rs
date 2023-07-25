use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    adapters::okx::Okx,
    event::EventType,
    interfaces::Adapter,
    models::{Exchange, Symbol},
};

pub type BatchId = i32;
pub type BackupId = i32;

pub struct AdapterSystem {
    exchange: Exchange,
    batch_dim: i32,
    backup_dim: i32,
    batch_id: BatchId,

    sub_count_map: BTreeMap<BatchId, i32>,
    map_batch_id_to_primary_backup_id: BTreeMap<BatchId, BackupId>,

    trade_subs: BTreeSet<String>,
    orderbook_subs: BTreeSet<String>,
    orderbook_snapshot_subs: BTreeSet<String>,

    pub map_orderbook_subs_to_batch_id: HashMap<Symbol, BatchId>,
    pub map_orderbook_snapshot_subs_to_batch_id: HashMap<Symbol, BatchId>,
    pub map_trade_subs_to_batch_id: HashMap<Symbol, BatchId>,

    pub adapter_map: BTreeMap<BatchId, Vec<Box<dyn Adapter>>>,
}

impl AdapterSystem {
    pub async fn new(exchange: Exchange, batch_dim: i32, backup_dim: i32) -> Self {
        let mut system = Self {
            exchange,
            batch_dim,
            backup_dim,
            batch_id: 0,
            sub_count_map: BTreeMap::new(),
            map_batch_id_to_primary_backup_id: BTreeMap::new(),
            trade_subs: BTreeSet::new(),
            orderbook_subs: BTreeSet::new(),
            orderbook_snapshot_subs: BTreeSet::new(),
            map_orderbook_subs_to_batch_id: HashMap::new(),
            map_orderbook_snapshot_subs_to_batch_id: HashMap::new(),
            map_trade_subs_to_batch_id: HashMap::new(),
            adapter_map: BTreeMap::new(),
        };

        system.batch_id += 1;
        let batch_id = system.batch_id;
        for backup_id in 0..system.backup_dim {
            system.register_adapter(&batch_id, backup_id).await;
        }

        system
    }

    /// Find where symbol for a certain event exists. If EventType is AdapterDisconnect then false
    fn subscribed(&self, symbol: &Symbol, kind: EventType) -> bool {
        match kind {
            EventType::Trade => self.trade_subs.get(symbol).is_some(),
            EventType::OrderbookUpdate => self.orderbook_subs.get(symbol).is_some(),
            EventType::OrderbookSnapshot => self.orderbook_snapshot_subs.get(symbol).is_some(),
            EventType::AdapterDisconnect => false,
        }
    }

    /// Returns a backup id
    fn primary_backup_id(&self, batch_id: &i32) -> BackupId {
        *self
            .map_batch_id_to_primary_backup_id
            .get(batch_id)
            .unwrap()
    }

    /// If you recieve none then either there isn't a symbol for the Trade or Orderbook. Or You've
    /// provided an invalid event: (OrderbookSnapshot, AdapterDisconnect)
    fn subscription_batch_id(&self, symbol: &Symbol, event: EventType) -> Option<BatchId> {
        match event {
            EventType::Trade => self.map_trade_subs_to_batch_id.get(symbol).copied(),
            EventType::OrderbookUpdate => self.map_orderbook_subs_to_batch_id.get(symbol).copied(),
            _ => None,
        }
    }
}

impl AdapterSystem {
    pub fn unsubscribe_trades(&mut self, symbol: &Symbol) {
        self.batch_id = *self.map_trade_subs_to_batch_id.get(symbol).unwrap();
        self.trade_subs.remove(symbol);

        // TODO: Tear down adapters if active subs = 0;
    }

    pub fn unsubscribe_orderbook(&mut self, symbol: &Symbol) {
        self.batch_id = *self.map_orderbook_subs_to_batch_id.get(symbol).unwrap();
        self.orderbook_subs.remove(symbol);

        // TODO: Tear down adapters if active subs = 0;
    }

    pub fn unsubscribe_orderbook_snapshot(&mut self, symbol: &Symbol) {
        self.batch_id = *self
            .map_orderbook_snapshot_subs_to_batch_id
            .get(symbol)
            .unwrap();
        self.orderbook_snapshot_subs.remove(symbol);

        // TODO: Tear down adapters if active subs = 0;
    }

    pub async fn resubscribe(&mut self, batch_id: BatchId, backup_id: BackupId) {
        println!("AdapterManagementSystem: Resubscribing {batch_id}  {backup_id}");

        while !self
            .adapter_map
            .get(&batch_id)
            .unwrap()
            .get(backup_id as usize)
            .unwrap()
            .connected()
        {}

        for (sym, id) in self.map_orderbook_subs_to_batch_id.iter() {
            if id == &self.batch_id {
                let symbol = sym.to_string();
                println!("AdapterManagementSystem: subscribing orderbook for {symbol}");
                self.adapter_map
                    .get_mut(&batch_id)
                    .unwrap()
                    .get_mut(backup_id as usize)
                    .unwrap()
                    .subscribe_orderbook(symbol)
                    .await;
            }
        }

        for (sym, id) in self.map_orderbook_snapshot_subs_to_batch_id.iter() {
            if id == &self.batch_id {
                let symbol = sym.to_string();
                println!("AdapterManagementSystem: subscribing orderbook snapshot for {symbol}");
                self.adapter_map
                    .get_mut(&batch_id)
                    .unwrap()
                    .get_mut(backup_id as usize)
                    .unwrap()
                    .subscribe_orderbook_snapshot(symbol)
                    .await;
            }
        }
        for (sym, id) in self.map_trade_subs_to_batch_id.iter() {
            if id == &self.batch_id {
                let symbol = sym.to_string();
                println!("AdapterManagementSystem: subscribing orderbook snapshot for {symbol}");
                self.adapter_map
                    .get_mut(&batch_id)
                    .unwrap()
                    .get_mut(backup_id as usize)
                    .unwrap()
                    .subscribe_trade(symbol)
                    .await;
            }
        }
    }
}

impl AdapterSystem {
    /// Completed
    pub async fn subscribe_orderbook_snapshot(&mut self, symbol: Symbol) {
        if self.orderbook_snapshot_subs.insert(symbol.clone()) {
            println!("AdapterManagementSystem: new orderbook snapshot subscription");

            if (self.sub_count_map.get(&self.batch_id).unwrap() >= &(&self.batch_dim - 1))
                || self.batch_id == -1
            {
                self.batch_id += 1;
                for backup_id in 0..self.backup_dim {
                    self.register_adapter(&backup_id, self.batch_id).await;
                }

                *self
                    .map_batch_id_to_primary_backup_id
                    .get_mut(&self.batch_id)
                    .unwrap() = 0;

                for backup_id in 0..self.batch_id {
                    self.adapter_map
                        .get_mut(&self.batch_id)
                        .unwrap()
                        .get_mut(self.batch_id as usize)
                        .unwrap()
                        .subscribe_orderbook_snapshot(symbol.clone())
                        .await;

                    println!("AdapterManagementSystem: at: {} - {} subscribing to orderbook snapshots for symbol: {}", self.batch_id, backup_id, symbol);
                }
            }
        } else {
            // Add to existing batch
            for backup_id in 0..self.batch_id {
                self.adapter_map
                    .get_mut(&self.batch_id)
                    .unwrap()
                    .get_mut(self.batch_id as usize)
                    .unwrap()
                    .subscribe_orderbook_snapshot(symbol.clone())
                    .await;

                println!("AdapterManagementSystem: at: {} - {} subscribing to orderbook snapshots for symbol: {}", self.batch_id, backup_id, symbol);
            }
            *self
                .map_orderbook_snapshot_subs_to_batch_id
                .get_mut(&symbol)
                .unwrap() = self.batch_id;
        }

        *self.sub_count_map.get_mut(&self.batch_id).unwrap() += 1;
    }
    /// Completed
    async fn register_adapter(&mut self, batch_id: &i32, backup_id: i32) {
        match self.exchange {
            Exchange::OKX => self
                .adapter_map
                .get_mut(batch_id)
                .unwrap()
                .push(Box::from(Okx::new(self.exchange).await)),
            // Exchange::BINANCEUSDM => todo!(),
            // Exchange::BINANCECOINM => todo!(),
            // Exchange::DERIBIT => todo!(),
            // Exchange::KRAKEN => todo!(),
            // Exchange::COINBASE => todo!(),
            // Exchange::HUOBI => todo!(),
            // Exchange::BITSTAMP => todo!(),
            // Exchange::BYBIT => todo!(),
            // Exchange::BITFINEX => todo!(),
            _ => todo!(),
        }

        println!("Picked");
        println!("Created");
        println!("AdapterManagementSystem: registered adapter to {batch_id} {backup_id}");

        while !self
            .adapter_map
            .get(batch_id)
            .unwrap()
            .get(backup_id as usize)
            .unwrap()
            .connected()
        {}

        println!(
            "AdapterManagementSystem: adapter at {batch_id} {backup_id} sucessfully connected"
        );
    }

    pub async fn subscribe_trades(&mut self, symbol: Symbol) {
        if self.trade_subs.insert(symbol.clone()) {
            println!("AdapterManagementSystem: New Trade Subscription");

            if self.sub_count_map.get(&self.batch_id).unwrap() >= &(&self.backup_dim - 1)
                || self.batch_id == -1
            {
                // make new batch
                self.batch_id += 1;
                for backup_id in 0..self.backup_dim {
                    self.register_adapter(&backup_id, self.batch_id).await;
                }

                *self
                    .map_batch_id_to_primary_backup_id
                    .get_mut(&self.batch_id)
                    .unwrap() = 0;

                for backup_id in 0..self.batch_id {
                    self.adapter_map
                        .get_mut(&self.batch_id)
                        .unwrap()
                        .get_mut(self.batch_id as usize)
                        .unwrap()
                        .subscribe_trade(symbol.clone())
                        .await;

                    println!(
                        "AdapterManagementSystem: at: {} - {} subscribing to trades for symbol: {}",
                        self.batch_id, backup_id, symbol
                    );
                }
            }
            *self.map_trade_subs_to_batch_id.get_mut(&symbol).unwrap() = self.batch_id;
        } else {
            // Add to existing batch
            for backup_id in 0..self.batch_id {
                self.adapter_map
                    .get_mut(&self.batch_id)
                    .unwrap()
                    .get_mut(self.batch_id as usize)
                    .unwrap()
                    .subscribe_trade(symbol.clone())
                    .await;

                println!(
                    "AdapterManagementSystem: at: {} - {} subscribing to trades for symbol: {}",
                    self.batch_id, backup_id, symbol
                );
            }
            *self.map_trade_subs_to_batch_id.get_mut(&symbol).unwrap() = self.batch_id;
        }

        *self.sub_count_map.get_mut(&self.batch_id).unwrap() += 1;
    }

    pub async fn subscribe_orderbook(&mut self, symbol: Symbol) {
        if self.orderbook_subs.insert(symbol.clone()) {
            println!("AdapterManagementSystem: New Orderbook Subscription");

            if self.sub_count_map.get(&self.batch_id).unwrap() >= &(&self.backup_dim - 1)
                || self.batch_id == -1
            {
                // make new batch
                self.batch_id += 1;
                for backup_id in 0..self.backup_dim {
                    self.register_adapter(&backup_id, self.batch_id).await;
                }

                *self
                    .map_batch_id_to_primary_backup_id
                    .get_mut(&self.batch_id)
                    .unwrap() = 0;

                for backup_id in 0..self.batch_id {
                    self.adapter_map
                        .get_mut(&self.batch_id)
                        .unwrap()
                        .get_mut(self.batch_id as usize)
                        .unwrap()
                        .subscribe_orderbook(symbol.clone())
                        .await;

                    println!(
                        "AdapterManagementSystem: at: {} - {} subscribing to orderbook for symbol: {}",
                        self.batch_id, backup_id, symbol
                    );
                }
            }
            *self
                .map_orderbook_subs_to_batch_id
                .get_mut(&symbol)
                .unwrap() = self.batch_id;
        } else {
            // Add to existing batch
            for backup_id in 0..self.batch_id {
                self.adapter_map
                    .get_mut(&self.batch_id)
                    .unwrap()
                    .get_mut(self.batch_id as usize)
                    .unwrap()
                    .subscribe_orderbook(symbol.clone())
                    .await;

                println!(
                    "AdapterManagementSystem: at: {} - {} subscribing to trades for symbol: {}",
                    self.batch_id, backup_id, symbol
                );
            }
            *self
                .map_orderbook_subs_to_batch_id
                .get_mut(&symbol)
                .unwrap() = self.batch_id;
        }

        *self.sub_count_map.get_mut(&self.batch_id).unwrap() += 1;
    }
}
