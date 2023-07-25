use awc;
use serde_json::Value;

const SYMBOLS_URLS: [&str; 8] = [
    "https://data.binance.com/api/v3/exchangeInfo",
    "https://api.bitfinex.com/v1/symbols_details",
    "https://api.exchange.coinbase.com/products",
    "https://api.kraken.com/0/public/AssetPairs",
    "https://api.bybit.com/spot/v3/public/symbols",
    "https://www.bitstamp.net/api/v2/trading-pairs-info/",
    "https://api.huobi.com/v1/common/symbols",
    "https://www.okx.com/api/v5/public/instruments?instType=SPOT",
];

pub async fn get_all_symbols(client: &awc::Client) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    let bin = get_bin_symbols(client).await;
    let coin = get_coin_symbols(client).await;
    let bitfinex = get_bitfinex_symbols(client).await;
    let kraken = get_kraken_symbols(client).await;
    let bybit = get_bybit_symbols(client).await;
    let bitstamp = get_bitstamp_symbols(client).await;
    let huobi = get_huobi_symbols(client).await;
    let okx = get_okx_symbols(client).await;
    // let deribit = get_deribit_symbols().await;
    // map.insert("deribit".to_string(), deribit.into());
    map.insert("binance".to_string(), bin.into());
    map.insert("coinbase".to_string(), coin.into());
    map.insert("bitfinex".to_string(), bitfinex.into());
    map.insert("bybit".to_string(), bybit.into());
    map.insert("bitstamp".to_string(), bitstamp.into());
    map.insert("kraken".to_string(), kraken.into());
    map.insert("huobi".to_string(), huobi.into());
    map.insert("okx".to_string(), okx.into());
    map
}
async fn get_bin_symbols(client: &awc::Client) -> Vec<String> {
    let mut bin_raw = client
        .get(SYMBOLS_URLS[0])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let arr = bin_raw.get_mut("symbols").unwrap().as_array_mut().unwrap();

    let symbols: Vec<String> = arr
        .iter()
        .map(|e| e.get("symbol").unwrap().as_str().unwrap().to_string())
        .collect();

    symbols
}

// #[tokio::test]
async fn get_coin_symbols(client: &awc::Client) -> Vec<String> {
    let mut coin_raw = client
        .get(SYMBOLS_URLS[2])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let arr = coin_raw.as_array_mut().unwrap();

    let symbols: Vec<String> = arr
        .iter()
        .map(|e| e.get("id").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

async fn get_bitfinex_symbols(client: &awc::Client) -> Vec<String> {
    let mut bitfinex_raw = client
        .get(SYMBOLS_URLS[1])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let arr = bitfinex_raw.as_array_mut().unwrap();
    // .to_owned();

    let symbols: Vec<String> = arr
        .iter()
        .map(|e| e.get("pair").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

async fn get_kraken_symbols(client: &awc::Client) -> Vec<String> {
    let mut kraken_raw = client
        .get(SYMBOLS_URLS[3])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let map = kraken_raw
        .get_mut("result")
        .unwrap()
        .as_object_mut()
        .unwrap();

    let symbols: Vec<String> = map
        .iter()
        .map(|(_, v)| v.get("wsname").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}
async fn get_bybit_symbols(client: &awc::Client) -> Vec<String> {
    let mut bybit_raw = client
        .get(SYMBOLS_URLS[4])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let map = bybit_raw
        .get_mut("result")
        .unwrap()
        .get_mut("list")
        .unwrap()
        .as_array_mut()
        .unwrap();
    // dbg!(&map);
    let symbols: Vec<String> = map
        .iter()
        .map(|v| v.get("name").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

async fn get_bitstamp_symbols(client: &awc::Client) -> Vec<String> {
    let bitstamp_raw = client
        .get(SYMBOLS_URLS[5])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let map = bitstamp_raw.as_array().unwrap();
    // dbg!(&map);
    let symbols: Vec<String> = map
        .iter()
        .map(|v| v.get("url_symbol").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

// #[tokio::test]
async fn get_huobi_symbols(client: &awc::Client) -> Vec<String> {
    let huobi_raw = client
        .get(SYMBOLS_URLS[6])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let map = huobi_raw.get("data").unwrap().as_array().unwrap();
    // dbg!(&map);
    let symbols: Vec<String> = map
        .iter()
        .map(|v| v.get("symbol").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

async fn get_okx_symbols(client: &awc::Client) -> Vec<String> {
    let okx_raw = client
        .get(SYMBOLS_URLS[7])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let map = okx_raw.get("data").unwrap().as_array().unwrap();
    // dbg!(&map);
    let symbols: Vec<String> = map
        .iter()
        .map(|v| v.get("instId").unwrap().as_str().unwrap().to_string())
        .collect();
    symbols
}

pub async fn store_symbols(
    path: Option<&std::path::Path>,
    name: Option<&std::path::Path>,
    client: &awc::Client,
) -> std::io::Result<()> {
    use std::path::Path;
    use tokio::fs;

    let mut dir_path = Path::new("./resources");
    if !dir_path.is_dir() {
        fs::create_dir(dir_path).await?;
    }
    let mut file_name = Path::new("symbols.json");

    if let Some(new_path) = path {
        dir_path = new_path;
    }

    if let Some(new_file) = name {
        file_name = new_file;
    }

    let map = get_all_symbols(client).await;

    let json = serde_json::to_value(map).expect("Literally a json object, WTF");

    let ref contents = serde_json::to_string_pretty(&json).expect("Is should already be json");

    fs::write(dir_path.join(file_name), contents).await?;

    Ok(())
}

pub async fn retrieve_symbols(
    path: Option<&std::path::Path>,
    name: Option<&std::path::Path>,
) -> std::io::Result<serde_json::Map<String, Value>> {
    use std::io::ErrorKind;
    use std::path::Path;
    use tokio::fs;

    let mut dir_path = Path::new("./resources");
    let mut file_name = Path::new("symbols.json");

    if let Some(new_path) = path {
        dir_path = new_path;
    }

    if let Some(new_file) = name {
        file_name = new_file;
    }

    if !dir_path.is_dir() {
        log::error!("Provided path isn't a existing directory");
        // fs::create_dir(dir_path).await?;
    }

    if !Path::new(dir_path).exists() {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            "Provided path doesn't exist",
        ));
    }

    let symbols_file = fs::read_to_string(dir_path.join(file_name)).await?;
    let map = serde_json::from_str::<serde_json::Map<String, Value>>(&symbols_file).unwrap();
    Ok(map)
}
