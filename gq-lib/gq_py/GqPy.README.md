# GqLive Python Library

GqLive is a Python library for receiving real-time cryptocurrency data from various exchanges, including Coinbase, Kraken, Bitfinex, and Binance.
Installation

You can install GqLive using pip:

```bash
$ pip install gq-live
```

## _Usage_

To use GqLive, you first need to create an instance of the GqLive class and connect to an exchange using the connect method. You can then read real-time data using the read method. Finally, you should disconnect from the exchange using the disconnect method.

Here's an example:

```python

import gq_live
from gq_live import Exchanges, AssetClass, DataType

# Connect to Coinbase and subscribe to BTC-USD ticker data
coinbase = gq_live
            .GqLive()
            .connect(Exchanges.Coinbase, AssetClass.Spot, DataType.Ticker, "BTC-USD")

# Read real-time data from Coinbase
while True:
    data = coinbase.read()
    print(data)

# Disconnect from Coinbase
coinbase.disconnect()
```

Supported Platforms

GqLive currently supports macOS. Support for Linux and Windows will be added in the future.
