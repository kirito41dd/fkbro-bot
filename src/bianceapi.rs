use reqwest::Client;
use tracing::trace;

pub struct BianceApi {
    client: Client,
    base_url: String,
}

impl BianceApi {
    pub fn new() -> Box<Self> {
        let client = Client::new();

        Box::new(BianceApi {
            client,
            base_url: "https://api.binance.com".into(),
        })
    }
    pub async fn tiker(
        &self,
        symbol: &str,
        window_size: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, "/api/v3/ticker"))
            .query(&[("symbol", symbol), ("windowSize", window_size)])
            .send()
            .await?;
        trace!("reqest {} code {}", resp.url(), resp.status());
        let data = resp.error_for_status()?.json::<serde_json::Value>().await?;
        Ok(data)
    }
}

pub mod types {}

#[cfg(test)]
mod test {
    use super::BianceApi;

    #[tokio::test]
    async fn test() {
        let api = BianceApi::new();
        let data = api.tiker("BTCUSDT", "1d").await.unwrap();
        println!("{:#?}", data);
    }
}
