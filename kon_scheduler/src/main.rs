use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use kon_rs::{
    algorithm::{IScheduleCallback, LiveInfo, RoomMatrix, Scheduler, SchedulerInfo, TaskInfo},
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

#[derive(Debug)]
struct CallbackData {
    progress_bar: Option<ProgressBar>,
    finished_task_count: usize,
    all_task_count: usize,
}
#[derive(Debug, Clone)]
struct ScheduleCallback {
    instance: Arc<Mutex<CallbackData>>,
}

impl ScheduleCallback {
    pub fn new() -> Self {
        Self {
            instance: Arc::new(Mutex::new(CallbackData {
                progress_bar: None,
                finished_task_count: 0,
                all_task_count: 0,
            })),
        }
    }
}

impl IScheduleCallback for ScheduleCallback {
    fn on_started(&mut self, scheduler_info: &SchedulerInfo) {
        let mut instance = self.instance.lock().unwrap();

        let spinner_style = ProgressStyle::with_template(&format!(
            "{{prefix:.bold}}▕{{bar:50.{}}}▏{{msg}}",
            "green"
        ))
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

        let pb = ProgressBar::new(scheduler_info.count as u64);
        pb.set_style(spinner_style);
        pb.set_prefix(format!("Run"));

        instance.progress_bar = Some(pb);
        instance.all_task_count = scheduler_info.count;
    }

    fn on_progress(&mut self, task_info: &TaskInfo) {
        let mut instance = self.instance.lock().unwrap();

        // instance.finished_task_count = instance.finished_task_count.max(task_info.finished_count);
        instance.finished_task_count += task_info.finished_count;

        let Some(progress_bar) = &instance.progress_bar else {
            return;
        };

        progress_bar.set_position(instance.finished_task_count as u64);
        progress_bar.set_message(format!(
            "{}/{}",
            instance.finished_task_count, instance.all_task_count
        ));
    }

    fn on_assigned(
        &mut self,
        table: &HashMap<BlockId, BandId>,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) {
        let instance = self.instance.lock().unwrap();
        let Some(progress_bar) = &instance.progress_bar else {
            return;
        };

        progress_bar.println("=============================================\n");

        for span_id in room_matrix.spans() {
            let mut string = String::new();
            for block_id in room_matrix.iter_span_blocks(*span_id) {
                let band_id = table.get(block_id).unwrap();
                let band_name = live_info.band_name(*band_id);
                string.push_str(&format!("{:?} ", band_name));
            }

            progress_bar.println(string);
        }
    }

    fn on_completed(&mut self) {
        let mut instance = self.instance.lock().unwrap();
        instance.progress_bar.as_mut().unwrap().finish();
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
