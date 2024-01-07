use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// ex. --band band_name;member0;member;
    #[arg(short = 'b', long = "band")]
    bands: Vec<String>,

    /// ex. --schedule band_name;true;false;false;true
    #[arg(short = 's', long = "schedule")]
    band_schedule: Vec<String>,
}

async fn run() {
    let args = Args::parse();
    println!("{:?}", args);
}

#[tokio::main]
async fn main() {
    run().await;
}
