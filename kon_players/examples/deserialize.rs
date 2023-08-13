use kon_players::clients::IClient;

fn main() {
    let data = kon_players::clients::SampleClient::default()
        .fetch()
        .unwrap();
    let members = kon_players::deserialize(&data);
    for member in members {
        println!("{:?}", member);
    }
}
