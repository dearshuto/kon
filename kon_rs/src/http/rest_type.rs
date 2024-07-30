use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContenType {
    pub id: String,
    pub title: String,
    pub body: BodyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HierarchyType {
    pub results: Vec<HierarchyResultType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HierarchyResultType {
    pub id: String,
    // pub children: Vec<ContenType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildPageType {
    pub id: String,
    pub title: String,
    pub body: BodyType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyType {
    pub view: ViewType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewType {
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct UserType {
    pub username: String,

    userKey: String,

    pub displayName: String,
}

#[cfg(test)]
mod tests {
    use crate::http::ContenType;

    #[test]
    fn deserialize() {
        let data = include_str!("../../res/rest.json");
        let content = serde_json::from_str::<ContenType>(data).unwrap();
        assert_eq!(content.id, "2765729402");
        assert_eq!(content.title, "20240406_春ライブ／合同練習会／時間割調整");
        assert_eq!(content.body.view.value, "<div></div>");
    }
}
