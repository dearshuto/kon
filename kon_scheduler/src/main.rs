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

    // 部屋割り
    let rooms: Vec<u32> = args.rooms.split('/').map(|x| x.parse().unwrap()).collect();

    // スケジュールを検索して...
    let scheduler = Scheduler::new();
    let assignment = scheduler.assign(&rooms, &live_info);
    let Ok(assignments) = assignment else {
        panic!();
    };

    // 先頭の結果を採用してみる
    if assignments.is_empty() {
        println!("No schedule...");
        return;
    }

    for assignment in assignments.iter().take(3) {
        println!("==============================");
        // 部屋割りを表示
        let i: Vec<(usize, usize)> = rooms
            .iter()
            .scan((0, 0), |(_start, end), room_count| {
                let start = *end;
                *end += *room_count;
                Some((start as usize, *end as usize))
            })
            .collect();
        for (start, end) in i {
            // 同時刻に割り振られたバンド数を取得
            let indices = &assignment[start..end];

            // バンド名に変換して表示
            let band_names: Vec<&str> = indices
                .iter()
                .map(|index| {
                    if live_info.band_ids().len() <= *index {
                        return "";
                    }

                    let id = live_info.band_ids()[*index];
                    let name = live_info.band_name(id);
                    name
                })
                .collect();
            println!("{:?}", band_names);
        }
    }
}

// ex. kon_scheduler --band name0/member0 --band name1/member0/member1 --band name2/member3 --rooms 2/1/2
// スケジュールは --schedule 遅い-早い
#[tokio::main]
async fn main() {
    run().await;
}
