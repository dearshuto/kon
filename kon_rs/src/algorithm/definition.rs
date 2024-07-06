use std::collections::HashMap;

use crate::{BandId, RoomId};

#[derive(Default)]
pub struct RoomMatrixBuilder {
    blocks: Vec<u8>,
}

impl RoomMatrixBuilder {
    pub fn build(self) -> RoomMatrix {
        // メモリー使用量で不利になるが、のワーストケースでバッファーを確保
        let width = self.blocks.len();
        let height = *self.blocks.iter().max().unwrap_or(&0) as usize;
        let capacity = width * height;

        // 部屋に
        let mut rooms = Vec::with_capacity(capacity);
        for y in 0..height {
            for x in 0..width {
                let room_capacity = self.blocks[x] as usize;

                // 部屋が利用可能なら Id を割り当てる
                if y < room_capacity {
                    rooms.push(Some(RoomId::new()));
                } else {
                    rooms.push(None);
                }
            }
        }

        // Id から部屋のインデックスを高速に検索するためのテーブル
        let room_table = rooms
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
                let Some(id) = id else {
                    return None;
                };

                Some((*id, index))
            })
            .collect();

        RoomMatrix {
            rooms,
            parralel_block_count: width,
            room_table,
        }
    }

    pub fn push_room(mut self, block_count: u8) -> Self {
        self.blocks.push(block_count);
        self
    }
}

pub struct RoomMatrix {
    //
    rooms: Vec<Option<RoomId>>,

    // 同時に利用可能な部屋数
    parralel_block_count: usize,

    room_table: HashMap<RoomId, usize>,
}

impl RoomMatrix {
    pub fn builder() -> RoomMatrixBuilder {
        RoomMatrixBuilder::default()
    }

    pub fn get(&self, id: RoomId) -> Option<(usize, usize)> {
        let Some(index) = self.room_table.get(&id) else {
            return None;
        };

        let x = index % self.parralel_block_count;
        let y = index / self.parralel_block_count;
        Some((x, y))
    }

    pub fn get_id(&self, x: usize, y: usize) -> Option<RoomId> {
        // 同時に利用できる部屋数の外側を指定されたら対応する部屋は常に見つからない
        if !(x < self.parralel_block_count) {
            return None;
        }

        let actual_index = x + self.parralel_block_count * y;
        let Some(id) = self.rooms.get(actual_index) else {
            return None;
        };

        let Some(id) = id else {
            return None;
        };

        Some(*id)
    }

    /// 割り当て可能な要素数
    pub fn len(&self) -> usize {
        self.room_table.len()
    }
}

pub struct Schedule {
    pub assign_table: HashMap<BandId, RoomId>,
}

#[cfg(test)]
mod tests {

    use super::RoomMatrix;

    #[test]
    fn room_matrix_simple() {
        let room_matrix = RoomMatrix::builder().push_room(1).build();
        assert_eq!(room_matrix.len(), 1);

        // 部屋の識別子を取得できる
        let id = room_matrix.get_id(0, 0).unwrap();
        assert!(room_matrix.get(id).unwrap() == (0, 0));

        // 範囲外の要素は識別子を取得できない
        assert!(room_matrix.get_id(1, 0).is_none());
        assert!(room_matrix.get_id(0, 1).is_none());
        assert!(room_matrix.get_id(1, 1).is_none());
    }
}
