// use anyhow::{Ok, Result, anyhow};
use clap::Parser;
mod auth;
mod google_cal;
mod time;

#[derive(Parser)]
struct Cli {
    tool: String,
    command: String,
    arguments: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.tool.as_str() {
        "gcal" => google_cal::get_calendar_service(&args.command, &args.arguments),
        _ => panic!("unknown tool! Please use one of: gcal"),
    }
    .await;

    println!(
        "tool: {:?}, command: {:?}, arguments: {:?}",
        args.tool, args.command, args.arguments
    )
}
