use std::sync::Arc;

use clap::Parser;
use futures::future::join_all;
use kon_rs::http::{Content, User};
use tokio::task::JoinHandle;
use url::Url;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// ex --base-url http://domain.com/confluence
    #[arg(long = "base-url", required = true)]
    base_url: String,

    /// ex. --user shikama_shuto --user dearshuto
    #[arg(long = "user", required = false)]
    users: Vec<String>,

    /// ex. --page 123456 --page 987654
    #[arg(long = "page", required = false)]
    pages: Vec<u64>,

    /// ex. --page 123456 --page 987654
    #[arg(long = "hierarchy", required = false)]
    hierarchy: Vec<u64>,
}

async fn run() {
    let args = Args::parse();

    let base_url = Url::parse(&args.base_url).unwrap();
    let confluence = Arc::new(kon_rs::http::Confluence::new(base_url));

    // ユーザー
    let user_request_join_handles: Vec<JoinHandle<User>> = args
        .users
        .into_iter()
        .map(|x| {
            tokio::spawn({
                let local = confluence.clone();
                async move { local.fetch_user(x).await }
            })
        })
        .collect();

    // ページ
    let page_request_join_handles: Vec<JoinHandle<Content>> = args
        .pages
        .into_iter()
        .map(|x| {
            tokio::spawn({
                let local = confluence.clone();
                async move { local.fetch_content(x).await }
            })
        })
        .collect();

    // 階層
    args.hierarchy
        .into_iter()
        .map(|x| {
            tokio::spawn({
                let local = confluence.clone();
                async move { local.fetch_child_page(x).await }
            })
        })
        .collect::<Vec<_>>();

    // ユーザーの出力
    let users = join_all(user_request_join_handles).await;
    for user in users {
        let user = user.unwrap();
        println!(
            "name: {}/display_name: {}",
            user.name(),
            user.display_name()
        );
    }

    // ページの出力
    let pages = join_all(page_request_join_handles).await;
    for page in pages {
        let page = page.unwrap();
        println!(
            "id={}, title={}, content={}",
            page.id(),
            page.title(),
            page.raw_content()
        );
    }
}

#[tokio::main]
async fn main() {
    run().await;
}
