use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    id: String,
    title: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    view: View,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct View {
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct User {
    username: String,

    userKey: u64,

    displayName: String,
}

#[cfg(test)]
mod tests {
    use super::Content;

    #[test]
    fn deserialize() {
        let data = include_str!("../res/rest.json");
        let content = serde_json::from_str::<Content>(data).unwrap();
        assert_eq!(content.id, "2765729402");
        assert_eq!(content.title, "20240406_春ライブ／合同練習会／時間割調整");
        assert_eq!(content.body.view.value, "<div></div>");
    }
}
