use clap::Parser;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{
    filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

mod bianceapi;
mod blockchairapi;
mod bot;

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
    let bot = bot::BotWrapper::new(args.token);
    bot.run().await;
}

fn init_tracing_subscriber() {
    let formatter = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true);
    let layer = tracing_subscriber::registry().with(
        filter::Targets::new()
            .with_target("fkbro_bot", LevelFilter::TRACE)
            .with_target("teloxide", LevelFilter::INFO)
            .and_then(tracing_subscriber::fmt::layer().event_format(formatter)),
    );
    layer.init()
}
