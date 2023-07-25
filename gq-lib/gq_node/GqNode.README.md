GqLive Node.js Library

This is a Node.js library for receiving real-time market data from various exchanges. It uses a Rust dynamic library for better performance.
Installation

```bash
$ npm install gq_node
```

## _Usage_

```typescript
import GqLive, { Exchanges, AssetClass, DataType } from "gq-live";


// Connect to the specified exchange and subscribe to the specified market data.
connect(exchange: Exchanges, assetClass: AssetClass, dataType: DataType, symbol: string): GqLive

// Read the next message from the socket.
read(): string

// Close the socket and release the resources.
disconnect(): void


// Example:
const coinbaseConn = new GqLive().connect(
  Exchanges.Coinbase,
  AssetClass.Spot,
  DataType.Trade,
  "BTC-USD"
);

while (true) {
  console.log(coinbaseConn.read());
}
```

## _Supported Platforms_:

- Macos: In development
- Linux: Planned
- Windows: Planned

This library is supported on Windows, macOS, and Linux. The corresponding Rust dynamic library files should be placed in the ../target/debug directory.
