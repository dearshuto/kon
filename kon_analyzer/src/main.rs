use clap::{Args, Parser, Subcommand};
use std::collections::{BTreeMap, HashSet};

// 「どの練習会」とかの検索対象のキーも引数で指定できるようにしたい
#[derive(Parser, Debug)]
struct CommandArgs {
    #[command(subcommand)]
    subcommand: SubCommandType,
}

#[derive(Debug, Subcommand)]
enum SubCommandType {
    WorkSheet(WorkSheetCommandArgs),

    Page(PageCommandArgs),
}

#[derive(Debug, Args)]
struct WorkSheetCommandArgs {
    #[arg(help = "アンケートを集計した csv ファイル")]
    input: String,
}

#[derive(Debug, Args)]
struct PageCommandArgs {
    #[command(subcommand)]
    subcommand: PageSubcommandType,
}

#[derive(Debug, Subcommand)]
enum PageSubcommandType {
    List(PageSubcommandListArgs),
}

#[derive(Debug, Args)]
struct PageSubcommandListArgs {
    #[clap(long)]
    #[arg(help = "ページ id")]
    id: u64,
}

fn execute_work_sheet_command(args: &WorkSheetCommandArgs) {
    // csv ロード
    // ファイルを csv として解釈できなければエラー
    let Ok(mut reader) = csv::Reader::from_path(&args.input) else {
        return;
    };

    // 各行を解析
    let mut records = Vec::default();
    for record in reader.deserialize() {
        let record: kon_rs::analyzer::Node = record.unwrap();
        records.push(record);
    }

    // 「第N回」をキーにして、参加者をバリューとする辞書を構築
    // 大小関係を保持する BTreeMap にしておけば N が支配的にソートされることを期待
    let table = records.iter().fold(
        BTreeMap::default(),
        |mut map: BTreeMap<String, HashSet<String>>, item| {
            // 参加予定がなければなにもしなくてよいので map をそのまま返す
            if !item.is_scheduled() {
                return map;
            }

            if let Some(value) = map.get_mut(item.time()) {
                // 要素が作られていたら結合していく
                for member in item.members() {
                    value.insert(member.to_string());
                }
            } else {
                // 要素が未生成なら新規追加
                map.insert(
                    item.time().to_string(),
                    item.members().iter().map(|x| x.to_string()).collect(),
                );
            }

            // 更新したインスタンスを返す
            map
        },
    );

    // 出力
    for (key, member_set) in table {
        // 参加者総数
        let sum_members = member_set.len();

        // 部員総数を母数とした出席率
        // 96 は 2024 年 10 月 1 日時の部員総数
        let ratio_percent = 100.0 * sum_members as f32 / 96.0;

        // 出力例：第1回: 30 (32.4%)
        println!("{key}: {sum_members} ({ratio_percent:2.1}%)");
    }
}

fn main() {
    let args = CommandArgs::parse();

    match args.subcommand {
        SubCommandType::WorkSheet(work_sheet_command_args) => {
            execute_work_sheet_command(&work_sheet_command_args)
        }
        SubCommandType::Page(_page_command_args) => todo!(),
    }
}
