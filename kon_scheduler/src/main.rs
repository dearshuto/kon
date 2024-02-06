use std::sync::Arc;

use clap::Parser;
use kon_rs::algorithm::{IScheduleCallback, Scheduler};

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

#[derive(Debug, Clone)]
struct ScheduleCallback {
    pub rooms: Vec<u32>,
}
impl IScheduleCallback for ScheduleCallback {
    fn assigned(&mut self, indicies: &[usize], live_info: &kon_rs::algorithm::LiveInfo) {
        let mut string = String::new();
        string.push_str("=============================================\n");

        // 部屋割りを表示
        let i: Vec<(usize, usize)> = self
            .rooms
            .iter()
            .scan((0, 0), |(_start, end), room_count| {
                let start = *end;
                *end += *room_count;
                Some((start as usize, *end as usize))
            })
            .collect();
        for (start, end) in i {
            let band_names: Vec<&str> = (start..end)
                .map(|index| {
                    if index >= indicies.len() {
                        return "";
                    }

                    let actual_index = indicies[index];
                    let id = live_info.band_ids()[actual_index];
                    let name = live_info.band_name(id);
                    name
                })
                .collect();

            string.push_str(&format!("{:?}\n", band_names));
        }

        println!("{}", string);
    }
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
    let band_schedule = args
        .band_schedule
        .iter()
        .map(|x| {
            let mut inputs = x.split('/');
            let band_name = inputs.next().unwrap().to_string();
            let schedule: Vec<bool> = inputs
                .map(|x| if x == "true" { true } else { false })
                .collect();
            (band_name, schedule)
        })
        .collect();
    let live_info = kon_rs::algorithm::create_live_info(&band_table, &band_schedule);
    let live_info = Arc::new(live_info);

    // 部屋割り
    let rooms: Vec<u32> = args.rooms.split('/').map(|x| x.parse().unwrap()).collect();

    // スケジュールを検索して...
    let mut callback = ScheduleCallback {
        rooms: rooms.to_vec(),
    };
    let scheduler = Scheduler::new();
    scheduler
        .assign_async_with_callback(&rooms, live_info.clone(), &mut callback)
        .await;
}

// ex. kon_scheduler --band name0/member0 --band name1/member0/member1 --band name2/member3 --rooms 2/1/2
// スケジュールは --schedule 遅い-早い
#[tokio::main]
async fn main() {
    run().await;
}
