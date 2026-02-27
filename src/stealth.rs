use tokio::time::{sleep, Duration};
use rand::Rng;

#[macro_export]
macro_rules! jitter {
    ($min:expr, $max:expr) => {{
        let delay = rand::thread_rng().gen_range($min..$max);
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }};
}

#[macro_export]
macro_rules! jitter_ms {
    ($min:expr, $max:expr) => {{
        let delay = rand::thread_rng().gen_range($min..$max);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    }};
}

#[macro_export]
macro_rules! with_jitter {
    ($min:expr, $max:expr, $action:expr) => {{
        jitter!($min, $max);
        $action
    }};
}

pub async fn legit_request(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        reqwest::Client::new().get(url).send()
    ).await;
    Ok(())
}

pub async fn random_legitimate_activity() {
    let urls = [
        "https://www.google.com/generate_204",
        "https://www.cloudflare.com/cdn-cgi/trace",
        "https://www.bing.com/favicon.ico",
        "https://www.microsoft.com/favicon.ico",
    ];

    let mut rng = rand::thread_rng();
    let url = urls[rng.gen_range(0..urls.len())];

    let _ = legit_request(url).await;
}
