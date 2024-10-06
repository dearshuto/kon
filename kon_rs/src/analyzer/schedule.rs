use std::collections::HashMap;

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize,
};

pub type FieldMap = HashMap<String, serde_json::Value>;

#[derive(Debug)]
pub struct Node {
    // バンド名
    band_name: String,

    // どの練習会
    time: String,

    // 参加者
    members: Vec<String>,

    // 出席予定か
    is_scheduled: bool,
}

impl Node {
    pub fn band_name(&self) -> &str {
        &self.band_name
    }

    /// 第何回の練習会か
    pub fn time(&self) -> &str {
        &self.time
    }

    /// メンバー名
    pub fn members(&self) -> &[String] {
        &self.members
    }

    /// 出席予定か
    pub fn is_scheduled(&self) -> bool {
        self.is_scheduled
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let field_map = deserializer.deserialize_map(NodeVisitor)?;

        // バンド名
        let Some(band_name_value) = field_map.get("バンド名") else {
            todo!();
        };
        let Some(band_name) = band_name_value.as_str() else {
            todo!();
        };

        // メンバー
        let Some(member_value) = field_map.get("当日参加者") else {
            todo!();
        };
        let Some(members) = member_value.as_str() else {
            todo!();
        };

        // どの練習会？
        let Some(times) = field_map.get("どの練習会？") else {
            todo!();
        };
        let Some(time) = times.as_str() else {
            todo!();
        };

        // 出席予定か
        let Some(scheduled_value) = field_map.get("参加しますか？") else {
            todo!();
        };
        let Some(scheduled_str) = scheduled_value.as_str() else {
            todo!();
        };

        Ok(Self {
            band_name: band_name.to_string(),
            time: time.to_string(),
            members: members.split(";").map(|x| x.to_string()).collect(),
            is_scheduled: scheduled_str == "TRUE",
        })
    }
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = FieldMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Node Visitor")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        // HashMapにバッファリングしていく。
        let mut field_map = FieldMap::new();
        while let Some(k) = map.next_key::<&str>()? {
            // valueはserde_json::Valueで取得するのが扱いやすい。
            let value = map.next_value()?;

            if field_map.insert(k.to_owned(), value).is_some() {
                // 重複したフィールドがあった場合はエラーにしておく。
                return Err(de::Error::custom(&format!("duplicate field `{k}`")));
            }
        }
        Ok(field_map)
    }
}
