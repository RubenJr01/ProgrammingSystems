use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use serde::Serialize;

// ----- Output format (as required) -----
#[derive(Clone, Serialize)]
struct WebsiteStatus {
    url: String,
    status_code: i32,           // HTTP code or -1 on network error
    response_time_ms: u128,     // elapsed in ms
    #[serde(with = "ts_seconds")]
    timestamp: SystemTime,      // when recorded
    error: Option<String>,      // error text if any
}

// Serialize SystemTime as unix seconds
mod ts_seconds {
    use serde::Serializer;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs = t
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        s.serialize_u64(secs)
    }
}

impl WebsiteStatus {
    fn print_row(&self) {
        let ts = self
            .timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        println!(
            "{:<40}  {:>3}  {:>6} ms  {}  {}",
            self.url,
            self.status_code,
            self.response_time_ms,
            ts,
            self.error.as_deref().unwrap_or("-")
        );
    }
}

#[derive(Clone)]
struct Config {
    workers: usize,
    timeout_ms: u64,
    max_retries: u32,
    json: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            workers: 16,
            timeout_ms: 5000,
            max_retries: 0,
            json: false,
        }
    }
}

fn parse_args() -> Config {
    let mut cfg = Config::default();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--workers" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) {
                    cfg.workers = v
                }
            }
            "-t" | "--timeout" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) {
                    cfg.timeout_ms = v
                }
            }
            "-r" | "--retries" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse().ok()) {
                    cfg.max_retries = v
                }
            }
            "--json" => cfg.json = true,
            _ => {}
        }
        i += 1;
    }
    cfg
}

fn check_once(url: &str, timeout_ms: u64) -> WebsiteStatus {
    let start = Instant::now();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(timeout_ms))
        .timeout_read(Duration::from_millis(timeout_ms))
        .timeout_write(Duration::from_millis(timeout_ms))
        .build();

    let mut status = -1;
    let mut err = None;

    match agent.get(url).call() {
        Ok(resp) => status = resp.status() as i32,
        Err(e) => err = Some(e.to_string()),
    }

    WebsiteStatus {
        url: url.to_string(),
        status_code: status,
        response_time_ms: start.elapsed().as_millis(),
        timestamp: SystemTime::now(),
        error: err,
    }
}

fn check_with_retries(url: &str, timeout_ms: u64, max_retries: u32) -> WebsiteStatus {
    let mut last = check_once(url, timeout_ms);
    let mut tries = 0;
    while tries < max_retries {
        // retry on network error or 5xx
        if last.error.is_none() && !(last.status_code >= 500 || last.status_code < 0) {
            break;
        }
        tries += 1;
        thread::sleep(Duration::from_millis(300));
        last = check_once(url, timeout_ms);
    }
    last
}

fn run_pool(urls: Vec<String>, workers: usize, timeout_ms: u64, retries: u32) -> Vec<WebsiteStatus> {
    use std::sync::{Arc, Mutex};
    let (tx, rx) = mpsc::channel::<WebsiteStatus>();
    let queue = Arc::new(Mutex::new(urls));
    let mut handles = Vec::new();

    for _ in 0..workers.max(1) {
        let tx = tx.clone();
        let q = Arc::clone(&queue);
        let handle = thread::spawn(move || loop {
            let url = {
                let mut lock = q.lock().unwrap();
                if lock.is_empty() {
                    break;
                }
                lock.pop().unwrap()
            };
            let res = check_with_retries(&url, timeout_ms, retries);
            let _ = tx.send(res);
        });
        handles.push(handle);
    }
    drop(tx);

    let mut out = Vec::new();
    while let Ok(item) = rx.recv() {
        out.push(item);
    }
    for h in handles {
        let _ = h.join();
    }
    out
}

fn main() {
    let cfg = parse_args();

    // URLs: either after a lone "--", or from stdin (one URL per line)
    let mut urls: Vec<String> = Vec::new();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Some(idx) = args.iter().position(|s| s == "--") {
        for s in args.into_iter().skip(idx + 1) {
            urls.push(s);
        }
    }
    if urls.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines().flatten() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') {
                urls.push(l.to_string());
            }
        }
    }

    if urls.is_empty() {
        eprintln!(
"USAGE (pick one):
  cat urls.txt | cargo run --release -- [options]
  cargo run --release -- [options] -- https://a.com https://b.com
Options:
  -w, --workers <N>   default 16
  -t, --timeout <ms>  default 5000
  -r, --retries <N>   default 0
  --json              print JSON (uses serde/serde_json)"
        );
        std::process::exit(2);
    }

    let results = run_pool(urls, cfg.workers, cfg.timeout_ms, cfg.max_retries);

    if cfg.json {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        println!(
            "{:<40}  {:>3}  {:>6}    {}  {}",
            "URL", "SC", "RT(ms)", "UNIX_TS", "ERROR"
        );
        println!("{}", "-".repeat(95));
        for r in results {
            r.print_row();
        }
    }
}
