use futures::StreamExt;
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::Message;
use std::collections::HashMap;
use std::{future::Future, io, pin::Pin, process::Output};
use tracing::{error, info, trace};
use tower::Service;

pub struct BotRequest {
   
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
    bot: teloxide::Bot,
    commands: HashMap<String, H>,
}

impl<H> BotWrapper<H> {
    pub fn new(token: String) -> Self {
        let bot = teloxide::Bot::new(token);
        BotWrapper {
            bot,
            commands: HashMap::new(),
        }
    }
    pub fn add_raw_handle(&mut self, key: String, h: H) {
        self.commands.insert(key, h);
    }

    pub async fn run(&self) {
       teloxide::repl(self.bot.clone(), |bot: Bot, msg: Message| async move {
        info!("msg {:#?}", msg);
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    }).await
    }
}
