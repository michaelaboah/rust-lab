// use crate::adapters::*;
use crate::models::{Exchange, Symbol};
pub struct InstrumentSystem {
    pub exchange: Exchange,
    instruments: Vec<Symbol>,
}

impl InstrumentSystem {
    pub fn new(exchange: Exchange) -> Self {
        Self {
            exchange,
            instruments: Vec::with_capacity(1000),
        }
    }

    pub fn pull_instruments(&self) {
        match self.exchange {
            Exchange::Okx => todo!("Need to implement code to request symbols from exchange"),
            // Exchange::BINANCEUSDM => todo!(),
            // Exchange::BINANCECOINM => todo!(),
            // Exchange::DERIBIT => todo!(),
            // Exchange::KRAKEN => todo!(),
            // Exchange::COINBASE => todo!(),
            // Exchange::HUOBI => todo!(),
            // Exchange::BITSTAMP => todo!(),
            // Exchange::BYBIT => todo!(),
            // Exchange::BITFINEX => todo!(),
            _ => eprintln!("InstrumentSystem: Exchange not supported"),
        };
    }

    pub fn peek_instruments(&self) -> &Vec<Symbol> {
        &self.instruments
    }
}
