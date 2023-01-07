use futures::{Future, StreamExt};

use lru_time_cache::LruCache;
use serde_json::{json, Value};
use teloxide::payloads::{SendMessage, SendMessageSetters};
use tokio::sync::Mutex;

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::format;

use std::pin::Pin;
use std::process::Output;
use std::sync::Arc;

use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::Dispatcher;

use teloxide::requests::Requester;
use teloxide::types::{Message, ParseMode, Poll, Update};
use teloxide::{utils::command::BotCommands, Bot};

use tracing::info;

use crate::blockchairapi;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "show btc transcation fee.")]
    Fee,
    #[command(description = "display this text.")]
    Help,
}

#[derive(Clone)]
pub struct BotWrapper {
    bot: teloxide::Bot,
    blockchairapi: Arc<blockchairapi::BlockChairApi>,
    tpl: Arc<tera::Tera>,
    cache1min: Arc<Mutex<LruCache<String, Arc<Value>>>>,
}

impl BotWrapper {
    pub fn new(token: String) -> Arc<Self> {
        let bot = teloxide::Bot::new(token);
        let blockchairapi = blockchairapi::BlockChairApi::new().into();
        let mut t = tera::Tera::new("templates/**/*.tera").unwrap();
        t.register_filter("fmt2f", filter_fmt2f);

        let cache1min = LruCache::with_expiry_duration(std::time::Duration::from_secs(60));
        BotWrapper {
            bot,
            blockchairapi,
            tpl: t.into(),
            cache1min: Arc::new(Mutex::new(cache1min)),
        }
        .into()
    }

    pub async fn run(&self) {
        self.bot
            .set_my_commands(Command::bot_commands())
            .await
            .unwrap();
        let handler = teloxide::dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(command_handler),
            )
            .branch(Update::filter_message().endpoint(message_handler));

        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(teloxide::dptree::deps![Arc::new(self.clone())])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    ctx: Arc<BotWrapper>,
) -> anyhow::Result<()> {
    //teloxide::utils::command::BotCommands
    info!(
        "msg {:#?}, cmd {:?}, {:?}",
        msg,
        cmd,
        Command::bot_commands()
    );
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Fee => ctx.anser_fee(msg).await?,
    };
    Ok(())
}

async fn message_handler(msg: Message, _bot: Bot) -> anyhow::Result<()> {
    info!("message {:#?}", msg);
    Ok(())
}

impl BotWrapper {
    async fn anser_fee(&self, msg: Message) -> anyhow::Result<()> {
        let data = self
            .get_1min_cache("fee", || self.blockchairapi.bitcoin_status())
            .await?;
        let mut ctx = tera::Context::new();
        ctx.insert("ad", "[骗局🤡都是骗局🔥](https://t.me/+WQk4D0iBkURk4iID)");
        ctx.insert("fee", data.as_ref());

        let str = self.tpl.render("fee.tera", &ctx)?;
        self.send_md_message(msg.chat.id, str).await?;
        Ok(())
    }

    fn send_md_message<C, T>(
        &self,
        chat_id: C,
        text: T,
    ) -> teloxide::requests::JsonRequest<SendMessage>
    where
        C: Into<teloxide::types::Recipient>,
        T: Into<String>,
    {
        self.bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true)
    }

    async fn get_1min_cache<F: Future<Output = Result<serde_json::Value, reqwest::Error>>>(
        &self,
        key: &str,
        get_val: impl FnOnce() -> F,
    ) -> Result<Arc<serde_json::Value>, reqwest::Error> {
        let mut gd = self.cache1min.lock().await;
        let data = gd.get(key);
        if let Some(d) = data {
            return Ok(d.clone());
        };
        drop(gd);

        let data = get_val().await?;
        let mut gd = self.cache1min.lock().await;
        gd.insert(key.into(), Arc::new(data));
        Ok(gd.get(key).unwrap().clone())
    }
}

fn filter_fmt2f(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let resutl = format!(
        "{:.2}",
        val.as_f64()
            .ok_or(tera::Error::msg(format!("not float: {}", val)))?
    );
    Ok(Value::String(resutl))
} /*  */
