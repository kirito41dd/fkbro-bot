use futures::Future;

use serde_json::Value;
use teloxide::payloads::{SendMessage, SendMessageSetters};

use tokio::sync::Mutex;
use tracing::log::trace;
use tracing::{info, trace_span, Instrument};
use ttl_cache::TtlCache;

use std::sync::Arc;
use std::time::Duration;

use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::Dispatcher;

use teloxide::requests::Requester;
use teloxide::types::{Message, ParseMode, Update};
use teloxide::{utils::command::BotCommands, Bot};

use crate::bianceapi::{self};
use crate::blockchairapi;
use crate::render::{
    filter_atof, filter_emoji, filter_escape_md, filter_fmt2f, filter_qoutevolume, filter_stob,
};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "price of coin.")]
    P(String),
    #[command(description = "show btc transcation fee.")]
    Fee,
    #[command(description = "address info.")]
    Addr(String),
    #[command(description = "help information.")]
    Help,
}

#[derive(Clone)]
pub struct BotWrapper {
    bot: teloxide::Bot,
    blockchairapi: Arc<blockchairapi::BlockChairApi>,
    bianceapi: Arc<bianceapi::BianceApi>,
    tpl: Arc<tera::Tera>,
    cache: Arc<Mutex<TtlCache<String, Arc<Value>>>>,
}

impl BotWrapper {
    pub fn new(token: String) -> Arc<Self> {
        let bot = teloxide::Bot::new(token);
        let blockchairapi = blockchairapi::BlockChairApi::new().into();
        let bianceapi = bianceapi::BianceApi::new().into();
        let mut t = tera::Tera::new("templates/**/*.tera").unwrap();
        t.register_filter("fmt2f", filter_fmt2f);
        t.register_filter("atof", filter_atof);
        t.register_filter("qoutevolume", filter_qoutevolume);
        t.register_filter("emoji", filter_emoji);
        t.register_filter("stob", filter_stob);
        t.register_filter("escape_md", filter_escape_md);

        let cache = ttl_cache::TtlCache::new(usize::MAX);
        BotWrapper {
            bot,
            blockchairapi,
            bianceapi,
            tpl: t.into(),
            cache: Arc::new(Mutex::new(cache)),
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
    let span = trace_span!("command", "{:?} {:?} {:?}", cmd, msg.chat.id, msg.id);
    let fut = async {
        let mut render_ctx = tera::Context::new();
        render_ctx.insert("ad", "[骗局🤡都是骗局🔥](https://t.me/+WQk4D0iBkURk4iID)");
        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Command::P(query) => ctx.anser_p(msg, query, render_ctx).await?,
            Command::Fee => ctx.anser_fee(msg, render_ctx).await?,
            Command::Addr(addr) => ctx.anser_addr(msg, &addr, render_ctx).await?,
        };
        Ok(())
    };
    fut.instrument(span).await
}

async fn message_handler(msg: Message, _bot: Bot) -> anyhow::Result<()> {
    trace!("message_handler msg {:?}", msg.text());
    Ok(())
}

impl BotWrapper {
    async fn anser_p(
        &self,
        msg: Message,
        mut query: String,
        mut render_ctx: tera::Context,
    ) -> anyhow::Result<()> {
        if query.is_empty() {
            query = "BTCUSDT".into();
        }
        if query.len() < 5 {
            query = format!("{}USDT", query)
        }
        query = query.to_uppercase();
        let data = self
            .get_from_cache(&format!("p:{}", query), Duration::from_secs(5), || {
                self.bianceapi.tiker(&query, "1d")
            })
            .await?;
        render_ctx.insert("data", data.as_ref());
        let str = self.tpl.render("p.tera", &render_ctx)?;
        self.send_md_message(msg.chat.id, str).await?;
        Ok(())
    }
    async fn anser_fee(&self, msg: Message, mut render_ctx: tera::Context) -> anyhow::Result<()> {
        let data = self
            .get_from_cache("fee", Duration::from_secs(60), || {
                self.blockchairapi.bitcoin_status()
            })
            .await?;
        render_ctx.insert("fee", data.as_ref());
        let str = self.tpl.render("fee.tera", &render_ctx)?;
        self.send_md_message(msg.chat.id, str).await?;
        Ok(())
    }

    async fn anser_addr(
        &self,
        msg: Message,
        addr: &str,
        mut render_ctx: tera::Context,
    ) -> anyhow::Result<()> {
        let data = self
            .get_from_cache(&format!("addr:{}", addr), Duration::from_secs(100), || {
                self.blockchairapi.bitcoin_addr_info(addr)
            })
            .await?;
        render_ctx.insert("data", data.as_ref());
        let str = self.tpl.render("addr.tera", &render_ctx)?;
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

    async fn get_from_cache<F: Future<Output = Result<serde_json::Value, reqwest::Error>>>(
        &self,
        key: &str,
        ttl: std::time::Duration,
        get_val: impl FnOnce() -> F,
    ) -> Result<Arc<serde_json::Value>, reqwest::Error> {
        let gd = self.cache.lock().await;
        let data = gd.get(key);
        if let Some(d) = data {
            return Ok(d.clone());
        };
        drop(gd);

        let data = get_val().await?;
        let mut gd = self.cache.lock().await;
        gd.insert(key.into(), Arc::new(data), ttl);
        Ok(gd.get(key).unwrap().clone())
    }
}
