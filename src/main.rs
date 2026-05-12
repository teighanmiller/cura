use anyhow::Ok;
mod auth;
use clap::{Parser, Subcommand};
mod google_cal;
mod time;
mod web;

#[derive(Subcommand, Clone)]
enum Tool {
    Gcal(google_cal::GcalArgs),
    Web(web::WebArgs),
}

#[derive(Parser)]
#[command(name = "cura")]
struct Cli {
    #[command(subcommand)]
    tool: Tool,
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    let results = match args.tool {
        Tool::Gcal(args) => google_cal::get_calendar_service(args).await?,
        Tool::Web(args) => web::websearch(args).await?,
    };

    println!("{}", results);
    Ok(())
}
