mod definition;
mod html_parser;
mod scheduler;

use std::collections::{HashMap, HashSet};

pub use definition::{RoomMatrix, Schedule};
pub use html_parser::HtmlParser;
pub use scheduler::Scheduler;

use crate::{BandId, UserId};

pub struct BandSchedule {
    pub name: String,
    pub is_available: Vec<bool>,
    pub members: Vec<String>,
}

pub struct LiveInfo {
    user_ids: Vec<UserId>,
    user_identifier_table: HashMap<UserId, String>,
    band_ids: Vec<BandId>,
    band_hash_table: HashMap<BandId, u128>,
    band_member_table: HashMap<BandId, Vec<UserId>>,
}

impl LiveInfo {
    pub fn user_ids(&self) -> &[UserId] {
        &self.user_ids
    }

    pub fn user_identifier(&self, id: UserId) -> Option<&str> {
        let Some(identifier) = self.user_identifier_table.get(&id) else {
            return None;
        };

        Some(identifier)
    }

    pub fn band_ids(&self) -> &[BandId] {
        &self.band_ids
    }

    pub fn band_hash(&self, id: BandId) -> Option<u128> {
        let Some(hash) = self.band_hash_table.get(&id) else {
            return None;
        };

        Some(*hash)
    }

    pub fn band_member_ids(&self, id: BandId) -> Option<&[UserId]> {
        let Some(user_ids) = self.band_member_table.get(&id) else {
            return None;
        };

        Some(user_ids)
    }
}

// band_table: バンド名 → メンバーたち
#[allow(dead_code)]
pub(crate) fn create_live_info(band_table: &HashMap<String, Vec<String>>) -> LiveInfo {
    // 重複と取り除いてユーザー一覧を生成
    let users: Vec<String> = {
        // 検索用のセット
        let mut name_set: HashSet<String> = HashSet::default();
        let mut users = Vec::default();

        for members in band_table.values() {
            for member in members {
                if !name_set.insert(member.to_string()) {
                    continue;
                }

                users.push(member.to_string());
            }
        }

        users.sort();
        users
    };

    // バンド名一覧（ソート済み）
    let bands = {
        let mut bands: Vec<String> = band_table.keys().map(|key| key.clone()).collect();
        bands.sort();
        bands
    };

    // ユーザー ID
    let user_ids: Vec<UserId> = (0..users.len()).map(|_| UserId::new()).collect();

    // ユーザー ID -> ユーザー識別子
    let user_identifier_table: HashMap<UserId, String> = user_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, users[index].clone()))
        .collect();

    // ユーザー識別子 -> ユーザー ID
    // 途中計算だけに使用してスコープを抜けたら消える
    let user_identifier_reverse_table: HashMap<String, UserId> = user_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (users[index].clone(), *id))
        .collect();

    // バンド ID
    let band_ids: Vec<BandId> = (0..bands.len()).map(|_| BandId::new()).collect();

    // バンド ID -> メンバー ID
    let band_member_table: HashMap<BandId, Vec<UserId>> = band_ids
        .iter()
        .enumerate()
        .map(|(index, band_id)| {
            let band_name = &bands[index];
            let names = band_table.get(band_name).unwrap();
            let member_ids: Vec<UserId> = names
                .iter()
                .map(|member| *user_identifier_reverse_table.get(member).unwrap())
                .collect();
            (*band_id, member_ids)
        })
        .collect();

    // バンドのハッシュ値
    let band_hash_table: HashMap<BandId, u128> = {
        // メンバーにビットを割り振る
        let member_hash_table: HashMap<UserId, u128> = user_ids
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let hash = 1 << index;
                (*id, hash)
            })
            .collect();

        // バンドに所属しているメンバーのビット和を算出
        // これをバンドのハッシュ値とする
        band_ids
            .iter()
            .map(|id| {
                let Some(member_ids) = band_member_table.get(id) else {
                    panic!();
                };

                let mut hash = 0;
                for member in member_ids {
                    let index = member_hash_table.get(member).unwrap();
                    hash |= index;
                }

                (*id, hash)
            })
            .collect()
    };

    LiveInfo {
        user_ids,
        user_identifier_table,
        band_ids,
        band_hash_table,
        band_member_table,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::create_live_info;

    #[test]
    fn simple() {
        let band_table = HashMap::from([
            (
                "a_band".to_string(),
                vec!["shikama_shuto".to_string(), "zzz".to_string()],
            ),
            ("b_band".to_string(), vec!["shikama_shuto".to_string()]),
        ]);
        let live_info = create_live_info(&band_table);

        let band_id_a = live_info.band_ids()[0];
        let band_id_b = live_info.band_ids()[1];
        let members_a = live_info.band_member_ids(band_id_a).unwrap();
        let members_b = live_info.band_member_ids(band_id_b).unwrap();

        assert_eq!(members_a.len(), 2);
        assert_eq!(members_b.len(), 1);

        let members: Vec<&str> = members_a
            .iter()
            .map(|id| live_info.user_identifier(*id).unwrap())
            .collect();
        assert!(members.contains(&"shikama_shuto"));
        assert!(members.contains(&"zzz"));
    }
}
