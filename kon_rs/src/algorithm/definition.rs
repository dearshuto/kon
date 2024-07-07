use std::collections::HashMap;

use crate::{BandId, BlockId, RoomId, SpanId};

#[derive(Default)]
pub struct RoomMatrixBuilder {
    blocks: Vec<u8>,
}

impl RoomMatrixBuilder {
    pub fn build(self) -> RoomMatrix {
        // 部屋に識別子を割り当てる
        let rooms: Vec<RoomId> = (0..self.blocks.len()).map(|_| RoomId::new()).collect();

        // 時間帯に識別子を割り当てる
        let spans: Vec<SpanId> = (0..*self.blocks.iter().max().unwrap_or(&0))
            .map(|_| SpanId::new())
            .collect();

        // 部屋の枠に識別子を割り当てる
        let room_block_table: HashMap<RoomId, Vec<BlockId>> = rooms
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let block_count = self.blocks[index];
                let block_ids: Vec<BlockId> = (0..block_count).map(|_| BlockId::new()).collect();
                (*id, block_ids)
            })
            .collect();

        // 枠の一覧を取得
        let mut blocks = Vec::default();
        for block_ids in room_block_table.values() {
            for block_id in block_ids {
                blocks.push(*block_id);
            }
        }

        // 時間帯ごとに枠をテーブル化
        let mut span_block_table: HashMap<SpanId, Vec<BlockId>> =
            spans.iter().map(|id| (*id, Vec::default())).collect();
        for y in 0..spans.len() {
            for x in 0..rooms.len() {
                let room_id = rooms[x];
                let Some(block_id) = room_block_table.get(&room_id).unwrap().get(y) else {
                    continue;
                };

                let span_id = spans[y];
                span_block_table.get_mut(&span_id).unwrap().push(*block_id);
            }
        }

        RoomMatrix {
            rooms,
            spans,
            blocks,
            room_block_table,
            span_block_table,
        }
    }

    pub fn push_room(mut self, block_count: u8) -> Self {
        self.blocks.push(block_count);
        self
    }
}

pub struct RoomMatrix {
    // 部屋
    rooms: Vec<RoomId>,

    // 時間帯
    spans: Vec<SpanId>,

    // 利用可能な枠に割り当てた識別子
    blocks: Vec<BlockId>,

    // 部屋単位で利用可能な枠
    // 歯抜けはない想定
    room_block_table: HashMap<RoomId, Vec<BlockId>>,

    // 時間帯で利用可能な枠
    span_block_table: HashMap<SpanId, Vec<BlockId>>,
}

impl RoomMatrix {
    pub fn builder() -> RoomMatrixBuilder {
        RoomMatrixBuilder::default()
    }

    /// 利用可能な部屋
    pub fn rooms(&self) -> &[RoomId] {
        &self.rooms
    }

    /// 利用可能な時間帯
    pub fn spans(&self) -> &[SpanId] {
        &self.spans
    }

    /// 割り当て可能な枠を取得します
    pub fn blocks(&self) -> &[BlockId] {
        &self.blocks
    }

    /// 指定した部屋で利用可能な枠
    pub fn iter_room_blocks(&self, room_id: RoomId) -> impl Iterator<Item = &BlockId> {
        self.room_block_table.get(&room_id).unwrap().iter()
    }

    /// 指定した時間帯で利用可能な枠
    pub fn iter_span_blocks(&self, span_id: SpanId) -> impl Iterator<Item = &BlockId> {
        self.span_block_table.get(&span_id).unwrap().iter()
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
        assert_eq!(room_matrix.blocks().len(), 1);
        assert_eq!(room_matrix.spans().len(), 1);
    }

    #[test]
    fn room_matrix_span() {
        let room_matrix = RoomMatrix::builder().push_room(2).build();
        let span_id = room_matrix.spans()[1];
        assert_eq!(room_matrix.iter_span_blocks(span_id).count(), 1);
    }

    #[test]
    fn room_matrix_multi_room() {
        let room_matrix = RoomMatrix::builder()
            .push_room(1)
            .push_room(2)
            .push_room(3)
            .build();

        let span_id_0 = room_matrix.spans()[0];
        let span_id_1 = room_matrix.spans()[1];
        let span_id_2 = room_matrix.spans()[2];
        assert_eq!(room_matrix.iter_span_blocks(span_id_0).count(), 3);
        assert_eq!(room_matrix.iter_span_blocks(span_id_1).count(), 2);
        assert_eq!(room_matrix.iter_span_blocks(span_id_2).count(), 1);
    }
}
