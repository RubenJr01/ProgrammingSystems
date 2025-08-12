use serde::Deserialize;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
enum ApiResult<T> {
    Success(T),
    ApiError(String),
    NetworkError(String),
}

trait Pricing {
    fn name(&self) -> &'static str;
    fn file_path(&self) -> &'static str;
    fn fetch_price(&self) -> ApiResult<f64>;
    fn save_to_file(&self, price: f64) -> Result<(), String> {
        let line = format!("{},{}\n", unix_seconds(), price);
        append_line(self.file_path(), &line)
    }
}

const UA: &str = "financial-fetcher/1.0";
const YF_GSPC_CHART: &str = "https://query2.finance.yahoo.com/v8/finance/chart/%5EGSPC";
const YF_BTC_CHART:  &str = "https://query2.finance.yahoo.com/v8/finance/chart/BTC-USD";
const YF_ETH_CHART:  &str = "https://query2.finance.yahoo.com/v8/finance/chart/ETH-USD";

#[derive(Deserialize)]
struct YahooChartResp {
    chart: ChartData,
}
#[derive(Deserialize)]
struct ChartData {
    result: Vec<ChartResult>,
}
#[derive(Deserialize)]
struct ChartResult {
    meta: ChartMeta,
}
#[derive(Deserialize)]
struct ChartMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
}

struct Bitcoin;
impl Pricing for Bitcoin {
    fn name(&self) -> &'static str { "Bitcoin" }
    fn file_path(&self) -> &'static str { "bitcoin.txt" }
    fn fetch_price(&self) -> ApiResult<f64> {
        fetch_yahoo_chart_price(YF_BTC_CHART, "BTC-USD")
    }
}

struct Ethereum;
impl Pricing for Ethereum {
    fn name(&self) -> &'static str { "Ethereum" }
    fn file_path(&self) -> &'static str { "ethereum.txt" }
    fn fetch_price(&self) -> ApiResult<f64> {
        fetch_yahoo_chart_price(YF_ETH_CHART, "ETH-USD")
    }
}

struct SP500;
impl Pricing for SP500 {
    fn name(&self) -> &'static str { "S&P 500" }
    fn file_path(&self) -> &'static str { "sp500.txt" }
    fn fetch_price(&self) -> ApiResult<f64> {
        fetch_yahoo_chart_price(YF_GSPC_CHART, "^GSPC")
    }
}

fn fetch_yahoo_chart_price(url: &str, tag: &str) -> ApiResult<f64> {
    match ureq::get(url).set("User-Agent", UA).call() {
        Ok(resp) => {
            if resp.status() == 200 {
                match resp.into_json::<YahooChartResp>() {
                    Ok(body) => match body.chart.result.get(0) {
                        Some(entry) => match entry.meta.regular_market_price {
                            Some(v) => ApiResult::Success(v),
                            None => ApiResult::ApiError(format!("{tag}: regular_market_price missing")),
                        },
                        None => ApiResult::ApiError(format!("{tag}: empty result array")),
                    },
                    Err(e) => ApiResult::ApiError(format!("{tag}: JSON parse error: {e}")),
                }
            } else {
                ApiResult::ApiError(format!("{tag}: HTTP {}", resp.status()))
            }
        }
        Err(e) => ApiResult::NetworkError(format!("{tag}: request failed: {e}")),
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn append_line(path: &str, line: &str) -> Result<(), String> {
    let mut f: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Open error ({path}): {e}"))?;
    f.write_all(line.as_bytes())
        .map_err(|e| format!("Write error ({path}): {e}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Financial Data Fetcher");
    println!("======================\n");
    println!("Endpoints:");
    println!("  ^GSPC -> {YF_GSPC_CHART}");
    println!("  BTC   -> {YF_BTC_CHART}");
    println!("  ETH   -> {YF_ETH_CHART}\n");

    let assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin),
        Box::new(Ethereum),
        Box::new(SP500),
    ];

    loop {
        for asset in assets.iter() {
            println!("Fetching {} …", asset.name());
            match asset.fetch_price() {
                ApiResult::Success(price) => {
                    println!("✅ Success: ${:.4}", price);
                    if let Err(e) = asset.save_to_file(price) {
                        eprintln!("❌ Save error ({}): {}", asset.name(), e);
                    } else {
                        println!("📝 Appended to {}", asset.file_path());
                    }
                }
                ApiResult::ApiError(e) => eprintln!("❌ API Error ({}): {}", asset.name(), e),
                ApiResult::NetworkError(e) => eprintln!("❌ Network Error ({}): {}", asset.name(), e),
            }
            println!();
        }
        thread::sleep(Duration::from_secs(10));
    }
}
