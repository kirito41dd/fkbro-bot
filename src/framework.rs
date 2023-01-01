use futures::StreamExt;
use std::collections::HashMap;
use std::{future::Future, io, pin::Pin, process::Output};
use telegram_bot::{update, GetMe};
use tracing::{error, info, trace};

use telegram_bot::Api;
use telegram_bot::Update;
use tower::Service;

pub struct BotRequest {
    pub api: Api,
    pub update: Update,
}
pub struct BotHandle;

impl Service<BotRequest> for BotHandle {
    type Response = ();

    type Error = io::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: BotRequest) -> Self::Future {
        todo!()
    }
}

pub struct BotWrapper<H> {
    api: Api,
    commands: HashMap<String, H>,
}

impl<H> BotWrapper<H> {
    pub fn new(token: String) -> Self {
        let api = Api::new(token);
        BotWrapper {
            api,
            commands: HashMap::new(),
        }
    }
    pub fn add_raw_handle(&mut self, key: String, h: H) {
        self.commands.insert(key, h);
    }

    pub async fn run(&self) {
        info!("bot run");
        let result = self.api.send(GetMe).await.unwrap();
        info!("get me {:?}", result);
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            info!("new update");
            match update {
                Ok(update) => {
                    trace!("update {:?}", update);
                }
                Err(err) => {
                    error!("update err:{}", err);
                }
            }
        }
    }
}
