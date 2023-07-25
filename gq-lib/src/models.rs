use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OHLCV {
    pub closePrice: f64,
    pub highPrice: f64,
    pub lowPrice: f64,
    pub openPrice: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Quote {
    pub askPrice: f64,
    pub askQty: f64,
    pub bidPrice: f64,
    pub bidQty: f64,
    pub percentPriceChange: f64,
    pub priceChange: f64,
    pub quoteVolume: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Time {
    pub closeTime: i64,
    pub eventTime: i64,
    pub gqPublished: String,
    pub openTime: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Trade {
    pub firstTradeId: i64,
    pub lastTradeQty: f64,
    pub numTrades: i64,
    pub weightedAvgPrice: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TickerData {
    pub assetClass: String,
    pub channel: String,
    pub exchange: String,
    pub ohlcv: OHLCV,
    pub quote: Quote,
    pub symbol: String,
    pub time: Time,
    pub trade: Trade,
    pub r#type: String,
}

#[cfg(test)]
mod tests_csv {
    use super::*;
    #[test]
    fn test_json_csv() {
        let mut wtr =
            csv::Writer::from_path(std::env::current_dir().unwrap().join("test.csv")).unwrap();
        let json = TickerData::default();
        wtr.serialize(json).unwrap();
        wtr.flush().unwrap();
    }

    #[test]
    fn test_write_csv_iter() {
        let mut wtr =
            csv::Writer::from_path(std::env::current_dir().unwrap().join("test-many.csv")).unwrap();
        let mut stacks = Vec::new();
        for _ in 0..12 {
            stacks.push(TickerData::default())
        }
        stacks.iter().for_each(|s| wtr.serialize(s).unwrap());
        wtr.flush().unwrap()
    }

    #[test]
    fn test_json_conversion() {
        let json_str = r#"{"assetClass":"spot","channel":"coinbase.spot.ohlcv.\u003cnil\u003e","exchange":"coinbase","ohlcv":{"closePrice":0,"highPrice":0,"lowPrice":0,"openPrice":0,"volume":0},"quote":{"askPrice":0,"askQty":0,"bidPrice":0,"bidQty":0},"symbol":"\u003cnil\u003e","time":{"gqPublished":"1678313957"},"trade":{"lastTradId":0},"type":"ohlcv"}"#;
        let ticker_data: TickerData = serde_json::from_str(json_str).unwrap();
        assert_eq!(json_str, serde_json::to_string(&ticker_data).unwrap())
    }
}
