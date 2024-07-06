use std::sync::{Arc, Mutex};

use clap::Parser;
use kon_rs::algorithm::{Evaluator, IScheduleCallback, RoomMatrix, Scheduler};

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

    /// make thread count 1 for debug
    #[arg(long, default_value_t = false)]
    force_synchronize_for_debug: bool,
}

#[derive(Debug, Clone)]
struct ScheduleCallback {
    pub rooms: Vec<u32>,
    pub score: Arc<Mutex<i32>>,
}
impl IScheduleCallback for ScheduleCallback {
    fn assigned(&mut self, indicies: &[usize], live_info: &kon_rs::algorithm::LiveInfo) {
        let mut string = String::new();
        let new_score = Evaluator::evaluate(&self.rooms, indicies, live_info) as i32;
        {
            let mut current_score = self.score.lock().unwrap();
            if *current_score >= new_score {
                return;
            }

            string.push_str("=============================================\n");
            string.push_str(&format!("Score {} -> {}\n", *current_score, new_score));

            *current_score = new_score;
        }

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

    // 部屋情報を構築
    // 今は未使用
    let mut room_matrix_builder = RoomMatrix::builder();
    for blocks in rooms {
        room_matrix_builder.push_room(blocks);
    }
    let _room_matrix = room_matrix_builder.build();

    // スケジュールを検索して...
    let mut callback = ScheduleCallback {
        rooms: rooms.to_vec(),
        score: Arc::new(Mutex::new(-1)),
    };
    let scheduler = Scheduler::new();
    if args.force_synchronize_for_debug {
        // 同期実行
        scheduler.assign_with_callback(&rooms, &live_info, &mut callback)
    } else {
        // 非同期実行
        scheduler
            .assign_async_with_callback(&rooms, live_info.clone(), &mut callback)
            .await;
    }
}

// ex. kon_scheduler --band name0/member0 --band name1/member0/member1 --band name2/member3 --rooms 2/1/2
// スケジュールは --schedule 遅い-早い
#[tokio::main]
async fn main() {
    run().await;
}
