use std::io::Write;

use clap::{Parser, Subcommand, value_parser};
use url::Url;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: CommandType,
}

#[derive(Debug, Subcommand)]
enum CommandType {
    Curl {
        #[arg(value_parser = value_parser!(Url))]
        addr: Url,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        CommandType::Curl { addr } => {
            let response = reqwest::get(addr).await.unwrap();
            if let Ok(str) = response.text().await {
                let mut stdout = std::io::stdout();
                stdout.write(str.as_bytes()).unwrap();
                stdout.flush().unwrap();
            }
        }
    }
}
