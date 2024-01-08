use std::sync::Arc;

use clap::Parser;
use kon_rs::algorithm::Scheduler;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// ex. --band band_name/member0/member
    #[arg(short = 'b', long = "band")]
    bands: Vec<String>,

    /// ex. --schedule band_name/true/false/false/true
    #[arg(short = 's', long = "schedule")]
    band_schedule: Vec<String>,

    /// ex. --rooms 1/2/1
    #[arg(short = 'r', long = "rooms")]
    rooms: String,
}

async fn run() {
    let args = Args::parse();

    // バンドと所属メンバー一覧
    let band_table = {
        let band_table = args
            .bands
            .iter()
            .map(|x| {
                let mut inputs = x.split('/');
                let band_name = inputs.next().unwrap().to_string();
                let members: Vec<String> = inputs.map(|x| x.to_string()).collect();
                (band_name, members)
            })
            .collect();
        band_table
    };

    let live_info = kon_rs::algorithm::create_live_info(&band_table);

    // 部屋割り
    let rooms: Vec<u32> = args.rooms.split('/').map(|x| x.parse().unwrap()).collect();

    // スケジュールを検索して...
    let scheduler = Scheduler::new();
    let assignment = scheduler.assign_async(&rooms, Arc::new(live_info)).await;
    let Ok(assignments) = assignment else {
        panic!();
    };

    // 先頭の結果を採用してみる
    let mut iterator = assignments[0].clone().into_iter();

    // 部屋割りを表示
    for room in rooms {
        // 同時刻に割り振られたバンド数を取得
        let indices = iterator.by_ref().take(room as usize);

        // バンド名に変換して表示
        let band_names: Vec<&str> = indices
            .map(|index| args.bands[index].split('/').next().unwrap())
            .collect();
        println!("{:?}", band_names);
    }
}

// ex. kon_scheduler --band name0/member0 --band name1/member0/member1 --band name2/member3 --rooms 2/1/2
#[tokio::main]
async fn main() {
    run().await;
}
