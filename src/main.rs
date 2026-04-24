use clap::{Parser, Subcommand};
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
async fn main() {
    let args = Cli::parse();

    match args.tool {
        Tool::Gcal {
            command,
            name,
            description,
            date,
            start_time,
            end_time,
        } => {
            google_cal::get_calendar_service(command, name, description, date, start_time, end_time)
                .await
        }
        Tool::Web {
            query,
            engine,
            max_value,
        } => web::websearch(query, engine, max_value).await,
    }
}
