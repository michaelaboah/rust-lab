export interface OHLC {
  closePrice: number;
  highPrice: number;
  lowPrice: number;
  openPrice: number;
  volume: number;
}

export interface Quote {
  askPrice: number;
  askQty: number;
  bidPrice: number;
  bidQty: number;
  percentPriceChange: number;
  priceChange: number;
  quoteVolume: number;
}

export interface MarketTime {
  closeTime: number;
  eventTime: number;
  gqPublished: string;
  openTime: number;
}

export interface Trade {
  firstTradeId: number;
  lastTradeQty: number;
  numTrades: number;
  weightedAvgPrice: number;
}

export default interface TickerData {
  _id: string;
  assetClass: string;
  channel: string;
  exchange: string;
  type: string;
  symbol: string;
  ohlcv: OHLC;
  quote: Quote;
  time: MarketTime;
  trade: TickerData;
}
