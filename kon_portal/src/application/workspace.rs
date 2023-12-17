use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use kon_rs::{
    algorithm::{RoomMatrix, Schedule, Scheduler},
    Band, InstrumentType, User,
};

use super::IClient;

struct SharedInstance<TClient>
where
    TClient: IClient + Sync + Send + 'static,
{
    user_ids: HashSet<String>,
    users: HashMap<String, User>,

    // 出演バンド
    bands: Option<Vec<Band>>,

    // 練習スケジュール
    schedule: Option<Schedule>,

    client: TClient,
}

pub struct Workspace<TClient>
where
    TClient: IClient + Sync + Send + 'static,
{
    // ユーザー一覧を取得するタスクのハンドル
    fetch_users_handle: tokio::task::JoinHandle<()>,

    // バンド一覧を取得するタスクのハンドル
    #[allow(dead_code)]
    fetch_bands_handle: tokio::task::JoinHandle<()>,

    join_handles: Vec<tokio::task::JoinHandle<()>>,

    // スレッド間で共有するオブジェクト
    shared_instance: Arc<Mutex<SharedInstance<TClient>>>,
}

impl<TClient> Workspace<TClient>
where
    TClient: IClient + Sync + Send + 'static,
{
    pub fn new(client: TClient) -> Self {
        let shared_instance = Arc::new(Mutex::new(SharedInstance {
            user_ids: HashSet::default(),
            users: HashMap::default(),
            bands: None,
            schedule: None,
            client,
        }));

        // 最初にユーザー一覧だけ取得しておく
        let task_instance = Arc::clone(&shared_instance);
        let handle = tokio::spawn(async move {
            let mut shared_instance = task_instance.lock().unwrap();
            let user_ids = shared_instance.client.fetch_users();
            for user_id in user_ids {
                shared_instance.user_ids.insert(user_id);
            }
        });

        // 出演バンド情報
        let task_instance = Arc::clone(&shared_instance);
        let fetch_band_task_handle = tokio::spawn(async move {
            let mut shared_instance = task_instance.lock().unwrap();
            let bands = shared_instance.client.fetch_bands();
            shared_instance.bands = Some(bands);
        });

        Self {
            fetch_users_handle: handle,
            fetch_bands_handle: fetch_band_task_handle,
            join_handles: Vec::default(),
            shared_instance,
        }
    }

    pub fn update(&mut self) {
        // ユーザー一覧が取得できなければ何もできないので待つ
        if !self.fetch_users_handle.is_finished() {
            return;
        }

        // 終了したタスクを抽出
        let finish_indicies = self
            .join_handles
            .iter()
            .enumerate()
            .filter_map(|(index, join_handle)| {
                if join_handle.is_finished() {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();
        // 削除対象のインデックスが変わらないように末尾から削除
        for index in finish_indicies.iter().rev() {
            self.join_handles.remove(*index);
        }

        let shared_instance = self.shared_instance.lock().unwrap();
        for user_id in &shared_instance.user_ids {
            if shared_instance.users.contains_key(user_id) {
                continue;
            }

            let task_instance = Arc::clone(&self.shared_instance);
            let task_id = user_id.clone();
            let handle = tokio::spawn(async move {
                let mut binding = task_instance.lock().unwrap();
                let Ok(user) = binding.client.fetch_user(&task_id) else {
                    return;
                };
                binding.users.insert(task_id, user);
            });

            self.join_handles.push(handle);
        }

        // スケジュールの構築
        if let Some(_bands) = &shared_instance.bands {
            if shared_instance.schedule.is_none() {
                let room_matrix = RoomMatrix::new(&[3, 2]);
                let _scheduler = Scheduler::new();
            }
        }
    }

    pub fn for_each_user_with_filter<TFunc: FnMut(&str, Option<&User>)>(
        &self,
        instrument_type: InstrumentType,
        mut func: TFunc,
    ) {
        let binding = self.shared_instance.lock().unwrap();
        for user_id in &binding.user_ids {
            let Some(user) = binding.users.get(user_id) else {
                continue;
            };

            if !user.instrument_type.contains(instrument_type) {
                continue;
            }
            func(user_id, Some(user));
        }
    }
}
