pub mod http;

pub struct User {
    pub name: String,
}

pub struct LiveProgram {
    // 出演者一覧
    pub user_names: Vec<String>,

    // 出演バンド
    pub item: Vec<String>,
}
