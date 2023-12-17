use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use kon_rs::User;

use super::IClient;

struct SharedInstance<TClient>
where
    TClient: IClient + Sync + Send + 'static,
{
    user_ids: HashSet<String>,
    users: HashMap<String, User>,
    client: TClient,
}

pub struct Workspace<TClient>
where
    TClient: IClient + Sync + Send + 'static,
{
    // ユーザー一覧を取得するタスクのハンドル
    fetch_users_handle: tokio::task::JoinHandle<()>,

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

        Self {
            fetch_users_handle: handle,
            join_handles: Vec::default(),
            shared_instance,
        }
    }

    pub fn update(&mut self) {
        // ユーザー一覧が取得できなければ何もできないので待つ
        if !self.fetch_users_handle.is_finished() {
            return;
        }

        // 終了したタスクを削除
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
        for index in finish_indicies {
            self.join_handles.remove(index);
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
    }
}
