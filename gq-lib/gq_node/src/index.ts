import ffi from "ffi-napi";
import ref from "ref-napi";
import TickerData from "../index";

const test_convert = ffi.Library("../target/debug/libgq_rust.dylib", {
  test_convert: ["void", ["string", "string", "string", "string"]],
}).test_convert;

test_convert("First", "Second", "Third", "Fourth");

export enum Exchanges {
  Coinbase = "coinbase",
  Kraken = "kraken",
  Bitfinex = "bitfinex",
  Binance = "binance",
}
export enum AssetClass {
  Spot = "spot",
}

export enum DataType {
  Ticker = "ticker",
  Book = "book",
  Trade = "trade",
}

export default class GqLive {
  private bufferPtr: ref.Pointer<unknown>;
  private osLibrary: string = "";

  private createSocket = ffi.Library(this.osLibrary, {
    create_socket: ["pointer", ["string", "string", "string", "string"]],
  }).create_socket;

  private receiveMessage = ffi.Library("../target/debug/libgq_rust.dylib", {
    receive_message: ["string", ["pointer"]],
  }).receive_message;

  constructor() {
    if (process.platform == "win32") {
      this.osLibrary = "../target/debug/libgq_rust.dll";
    } else if (process.platform == "darwin") {
      this.osLibrary = "../target/debug/libgq_rust.dylib";
    } else if (process.platform == "linux") {
      this.osLibrary = "../target/debug/libgq_rust.so";
    }
  }

  connect(
    exchange: Exchanges,
    assetClass: AssetClass,
    dataType: DataType,
    symbol: string
  ): this {
    this.bufferPtr = this.createSocket(exchange, assetClass, dataType, symbol);
    return this;
  }

  read(): string {
    return this.receiveMessage(this.bufferPtr);
  }
}

const coinbaseConn = new GqLive().connect(
  Exchanges.Coinbase,
  AssetClass.Spot,
  DataType.Trade,
  "BTC-USD"
);

while (true) {
  console.log(coinbaseConn.read());
}
