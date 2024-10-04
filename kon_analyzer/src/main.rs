use std::collections::{HashMap, HashSet};
use clap::Parser;

// 「どの練習会」とかの検索対象のキーも引数で指定できるようにしたい
#[derive(Parser, Debug)]
struct Args
{
    /// 入力ファイル (.csv) のパス
    #[arg(long, required = true)]
    input: String,

    /// 参加者が格納されているキーキー（"当日参加者"とか）
    #[arg(long, default_value_t = default_target())]
    target: String,
}

fn default_target() -> String {
    String::from("当日参加者")
}

// ハードコードをなくしたい
fn main() {
    let args = Args::parse();

    let Ok(mut reader) = csv::Reader::from_path(args.input) else {
        return;
    };

    // キーだけ抽出
    let keys: Vec<String> = reader.headers().unwrap().iter().map(|x| x.to_string()).collect();

    // 「どの練習会？」のキーのインデックス
    let volume_index = keys.iter().position(|x| x == "どの練習会？").unwrap();

    // 当日の参加者のインデックス
    let members_index = keys.iter().position(|x| x == &args.target).unwrap();

    // 「参加しますか？」のインデックス
    let attend_index = keys.iter().position(|x| x == "参加しますか？").unwrap();

    // 「第N回」をキーとしてその練習会に参加したメンバーを参照するテーブル
    // メンバーは重複を無視するのでデータ型は HashSet としておく
    let mut table: HashMap<String, HashSet<String>> = HashMap::new();

    // csv データを 1 行ずつ走査
    for result in reader.records() {
        let Ok(record) = result else {
            continue;
        };

        let volume = record.get(volume_index).unwrap();
        let members = record.get(members_index).unwrap();

        // 新規追加ならテーブルを用意
        if !table.contains_key(volume) {
            table.insert(volume.to_string(), Default::default());
        }

        // 不参加なら参加者としてカウントしない
        let is_available = record.get(attend_index).unwrap() == "TRUE";
        if !is_available
        {
            continue;
        }

        // 参加者一覧は ; で区切られてるのでパースする
        let members = members.split(";").map(|x| x.to_string());
        table.get_mut(volume).unwrap().extend(members);
    }

    // 出力
    for (key, value) in table {
        // 出力例：第1回: 30 (32.4%)
        println!("{}: {} ({:2.1})", key, value.len(), 100.0 * value.len() as f32 / 96.0);
    }
}
