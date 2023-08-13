pub trait IClient {
    fn fetch(&mut self) -> Result<String, Box<dyn std::error::Error>>;
}

// 決まったデータを返すクライアント
#[derive(Default)]
pub struct SampleClient;

impl IClient for SampleClient {
    // 常に成功する
    fn fetch(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = include_str!("example.csv");
        Ok(data.to_string())
    }
}
