use std::fmt::format;

use clap::Parser;
use telegram_bot::{GetMe, Api};
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{
    filter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

mod framework;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    token: String,
}

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    let args = Args::parse();
    info!("Hello, world!");
    let mut bot = framework::BotWrapper::new(args.token);
    bot.add_raw_handle("key".into(), framework::BotHandle);
    bot.run().await;
}

fn init_tracing_subscriber() {
    let formatter = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true);
    let layer = tracing_subscriber::registry().with(
        filter::Targets::new()
            .with_target("fkbro_bot", LevelFilter::TRACE)
            .with_target("telegram_bot", LevelFilter::TRACE)
            .and_then(tracing_subscriber::fmt::layer().event_format(formatter.clone())),
    ).with(tracing_subscriber::fmt::layer().event_format(formatter));
    layer.init()
}
