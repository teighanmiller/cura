use anyhow::Ok;
mod auth;
use clap::{Parser, Subcommand};
mod google_cal;
mod time;
mod web;

#[derive(Subcommand, Clone)]
enum Tool {
    /// Interact with Google Calendar
    Gcal(google_cal::GcalArgs),
    /// Search the web
    Web(web::WebArgs),
}

#[derive(Parser)]
#[command(name = "cura", about = "Your personal assistant CLI")]
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
