use reqwest::Client;
use tracing::trace;

pub struct BlockChairApi {
    client: Client,
    base_url: String,
}

impl BlockChairApi {
    pub fn new() -> Self {
        let client = Client::new();

        BlockChairApi {
            client,
            base_url: "https://api.blockchair.com".into(),
        }
    }

    pub async fn bitcoin_status(&self) -> Result<serde_json::Value, reqwest::Error> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, "/bitcoin/stats"))
            .send()
            .await?;
        trace!("reqest {} code {}", resp.url(), resp.status());
        let data = resp.error_for_status()?.json::<serde_json::Value>().await?;
        Ok(data)
    }
}
