use anyhow::Ok;
use clap::{Parser, Subcommand};
use websearch::SearchError;
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

enum CliError {
    Web(SearchError),
    Calendar(google_calendar3::Error),
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
        } => {
            google_cal::get_calendar_service(command, name, description, date, start_time, end_time)
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
