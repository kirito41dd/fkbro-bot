use reqwest::Client;
use tracing::trace;

pub struct BlockChairApi {
    client: Client,
    base_url: String,
}

impl BlockChairApi {
    pub fn new() -> Box<Self> {
        let client = Client::new();

        Box::new(BlockChairApi {
            client,
            base_url: "https://api.blockchair.com".into(),
        })
    }

    pub async fn bitcoin_status(&self) -> Result<serde_json::Value, reqwest::Error> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, "/bitcoin/stats"))
            .send()
            .await?;
        trace!("reqest {} code {}", resp.url(), resp.status());
        let data = resp.json::<serde_json::Value>().await?;
        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use crate::blockchairapi::BlockChairApi;

    #[tokio::test]
    async fn test_bitcoin_status() {
        let api = BlockChairApi::new();
        let data = api.bitcoin_status().await.unwrap();
        println!("{:#?}", data);
    }
}
