use std::{collections::HashMap, sync::Arc};

use kon_rs::algorithm::{create_live_info, RoomMatrix, Scheduler};

#[test]
fn simple() {
    // 1 部屋 3 枠
    // 3! 分だけ結果がある
    let room_matrix = RoomMatrix::builder().push_room(3).build();
    let band_table = HashMap::from([
        ("band_x".to_string(), vec!["a".to_string()]),
        ("band_y".to_string(), vec!["a".to_string()]),
        ("band_z".to_string(), vec!["b".to_string()]),
    ]);
    let band_schedule: HashMap<String, Vec<bool>> = band_table
        .keys()
        .map(|key| (key.to_string(), vec![true; 3]))
        .collect();
    let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

    let scheduler = Scheduler::new();
    let result = scheduler.assign(&room_matrix, &live_info);
    assert_eq!(result.len(), 6);
}

#[test]
fn simple_parallel() {
    // 2 部屋で 2枠と 1 枠
    // Room0 | Room1
    //  ○    | ○
    //  ○    | ×
    //
    // band_x と band_y はメンバーの衝突によって同時刻に入れない
    let room_matrix = RoomMatrix::builder().push_room(2).push_room(1).build();
    let band_table = HashMap::from([
        ("band_x".to_string(), vec!["a".to_string()]),
        ("band_y".to_string(), vec!["a".to_string()]),
        ("band_z".to_string(), vec!["b".to_string()]),
    ]);
    let band_schedule: HashMap<String, Vec<bool>> = band_table
        .keys()
        .map(|key| (key.to_string(), vec![true; 3]))
        .collect();
    let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

    let scheduler = Scheduler::new();
    let result = scheduler.assign(&room_matrix, &live_info);
    assert_eq!(result.len(), 4);
}

#[test]
fn simple_none() {
    // 3 部屋 各 1 枠ずつ
    // 枠数は足りているけど band_x と band_y は同時刻に入れないので解なし
    let room_matrix = RoomMatrix::builder()
        .push_room(1)
        .push_room(1)
        .push_room(1)
        .build();
    let band_table = HashMap::from([
        ("band_x".to_string(), vec!["a".to_string()]),
        ("band_y".to_string(), vec!["a".to_string()]),
        ("band_z".to_string(), vec!["b".to_string()]),
    ]);
    let band_schedule: HashMap<String, Vec<bool>> = band_table
        .keys()
        .map(|key| (key.to_string(), vec![true; 3]))
        .collect();
    let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

    let scheduler = Scheduler::new();
    let result = scheduler.assign(&room_matrix, &live_info);
    assert_eq!(result.len(), 0);
}

#[test]
fn simple_() {
    // 3 部屋 各 1 枠ずつ
    // 枠数は足りているけど band_x と band_y は同時刻に入れないので解なし
    let room_matrix = RoomMatrix::builder().push_room(5).build();
    let room_matrix = Arc::new(room_matrix);
    let band_table = HashMap::from([
        ("band_x".to_string(), vec!["a".to_string()]),
        ("band_y".to_string(), vec!["b".to_string()]),
        ("band_z".to_string(), vec!["c".to_string()]),
        ("band_u".to_string(), vec!["d".to_string()]),
        ("band_v".to_string(), vec!["e".to_string()]),
    ]);
    let band_schedule: HashMap<String, Vec<bool>> = band_table
        .keys()
        .map(|key| (key.to_string(), vec![true; 5]))
        .collect();
    let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);
    let live_info = Arc::new(live_info);

    let scheduler = Scheduler::new();
    let result = scheduler.assign(&room_matrix, &live_info);
    assert_eq!(result.len(), 120);

    // 並列実行
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            let scheduler = Scheduler::new();
            let result = scheduler.assign_async(room_matrix, live_info).await;
            assert_eq!(result.len(), 120);
        });
}
