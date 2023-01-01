use clap::Parser;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{
    filter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};
use teloxide::{Bot,utils::command::BotCommands, types::Message, requests::{ResponseResult, Requester}, dispatching::repls::CommandReplExt};

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
    let bot = Bot::new(args.token);
    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone,Debug)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    info!("msg {:#?}, cmd {:?}", msg, cmd);
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(msg.chat.id, format!("Your username is @{username} and age is {age}."))
                .await?
        }
    };
    Ok(())
}

fn init_tracing_subscriber() {
    let formatter = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true);
    let layer = tracing_subscriber::registry().with(
        filter::Targets::new()
            .with_target("fkbro_bot", LevelFilter::TRACE)
            .with_target("teloxide", LevelFilter::INFO)
            .and_then(tracing_subscriber::fmt::layer().event_format(formatter.clone())),
    );
    layer.init()
}
