use anyhow::Ok;
use clap::{Parser, Subcommand};

use crate::google_cal::SeriesArgs;
mod auth;
mod google_cal;
mod time;
mod web;

#[derive(Subcommand, Clone)]
enum Tool {
    Gcal {
        command: google_cal::GcalCommands,
        name: Option<String>,
        description: Option<String>,
        date: Option<String>,
        start_time: Option<String>,
        end_time: Option<String>,
        freq: Option<SeriesArgs>,
    },
    Web {
        query: String,
        #[arg(short, long)]
        engine: Option<web::SearchEngine>,
        #[arg(long)]
        max_value: Option<u32>,
    },
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
        Tool::Gcal {
            command,
            name,
            description,
            date,
            start_time,
            end_time,
            freq,
        } => {
            google_cal::get_calendar_service(
                command,
                name,
                description,
                date,
                start_time,
                end_time,
                freq,
            )
            .await?
        }
        Tool::Web {
            query,
            engine,
            max_value,
        } => web::websearch(query, engine, max_value).await?,
    };

    println!("{}", results);
    Ok(())
}
