use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kon_rs::{
    algorithm::{
        IScheduleCallback, LiveInfo, RoomMatrix, Scheduler, SchedulerInfo, TaskId, TaskInfo,
    },
    BandId, BlockId,
};

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
    multi_progress: MultiProgress,
    progress_bar: Option<ProgressBar>,
}

impl ScheduleCallback {
    pub fn new() -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            progress_bar: None,
        }
    }
}

impl IScheduleCallback for ScheduleCallback {
    fn on_started(&mut self, scheduler_info: &SchedulerInfo) {
        let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        self.multi_progress
            .println(format!("Iterate Count: {}", scheduler_info.count))
            .unwrap();
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!("[{}/{}]", 1, scheduler_info.count));

        self.progress_bar = Some(pb);
    }

    fn on_progress(&mut self, _task_id: TaskId, _task_info: &TaskInfo) {}

    fn on_assigned(
        &mut self,
        table: &HashMap<BlockId, BandId>,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) {
        println!("=============================================\n");

        for span_id in room_matrix.spans() {
            let mut string = String::new();
            for block_id in room_matrix.iter_span_blocks(*span_id) {
                let band_id = table.get(block_id).unwrap();
                let band_name = live_info.band_name(*band_id);
                string.push_str(&format!("{:?} ", band_name));
            }
            println!("{}", string);
        }
    }

    fn on_completed(&mut self) {}
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

    // 部屋割り
    let rooms: Vec<u32> = args.rooms.split('/').map(|x| x.parse().unwrap()).collect();

    // 部屋情報を構築
    let mut room_matrix_builder = RoomMatrix::builder();
    for blocks in &rooms {
        room_matrix_builder = room_matrix_builder.push_room(*blocks as u8);
    }
    let room_matrix = room_matrix_builder.build();

    let live_info = kon_rs::algorithm::create_live_info(&band_table, &band_schedule, &room_matrix);
    let live_info = Arc::new(live_info);

    // スケジュールを検索して...
    let callback = ScheduleCallback::new();
    let mut scheduler = Scheduler::new_with_callback(callback);
    if args.force_synchronize_for_debug {
        // 同期実行
        scheduler.assign(&room_matrix, &live_info)
    } else {
        // 非同期実行
        scheduler
            .assign_async(Arc::new(room_matrix), live_info)
            .await;
    }
}

// ex. kon_scheduler --band name0/member0 --band name1/member0/member1 --band name2/member3 --rooms 2/1/2
// スケジュールは --schedule 遅い-早い
#[tokio::main]
async fn main() {
    run().await;
}
