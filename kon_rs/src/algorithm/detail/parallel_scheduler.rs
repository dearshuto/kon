use std::{ops::Range, sync::Arc, thread, time::Duration};

use crate::algorithm::{
    pruning_decorators::{
        BandScheduleTraverseDecorator, ITraverseDecorator, MemberConflictTraverseDecorator,
        TreeTraverser,
    },
    IParallelTreeCallback, LiveInfo,
};

use super::PartialPermutation;

pub struct ParallelScheduler;

impl ParallelScheduler {
    pub async fn assign<T>(rooms: &[u32], live_info: Arc<LiveInfo>, callback: &mut T)
    where
        T: IParallelTreeCallback + Clone + Send + 'static,
    {
        let room_assign: Vec<Range<usize>> = rooms
            .iter()
            .scan((0, 0), |(_start, end), room_count| {
                let start = *end;
                *end += *room_count;
                Some((start as usize, *end))
            })
            .map(|(start, end)| Range {
                start,
                end: end as usize,
            })
            .collect();

        let context = Arc::new(ScheduleContext {
            live_info,
            room_assign,
        });
        let band_count = context.live_info().band_ids().len();

        let mut tasks: Vec<tokio::task::JoinHandle<TaskResult>> = Vec::new();

        // 末尾 15 桁で部分集合を作成
        // 15 は CPU をほどよく占有できるだけの計算量になる経験則
        let start = band_count.max(10) - 10;
        let mut partial_permutation = PartialPermutation::new(band_count, start);

        // 最初の部分木。これだけループの外に置くのがダサいと思いつつ...
        let task = Self::spawn_taks(
            PartialPermutation::new(band_count, start),
            context.clone(),
            callback.clone(),
        );
        tasks.push(task);

        // はじめに一定数のタスクを作成しておく
        const TASK_COUNT_MAX: usize = 64;
        for _index in 0..TASK_COUNT_MAX {
            let Some(next) = partial_permutation.next_part() else {
                break;
            };

            // タスクを作成
            let task = Self::spawn_taks(next, context.clone(), callback.clone());
            tasks.push(task);

            // 部分を更新して次のループへ
            partial_permutation = partial_permutation.next_part().unwrap();
        }

        // 毎秒タスクの進捗をポーリング
        while !tasks.is_empty() {
            thread::sleep(Duration::from_millis(16));

            // ベクターの要素を削除しながらループをまわすので最後から走査する
            for index in (0..tasks.len()).rev() {
                // 作業中ならなにもしない
                if !tasks[index].is_finished() {
                    continue;
                }

                // ハンドルを管理下から外す
                // 念のため await してるけどなくてもいいかも
                let finished_task = tasks.swap_remove(index);
                let task_result = finished_task.await.unwrap();

                // 部分木の更新中に別の部分木までスキップする可能性がある
                // タスクとして発行済みの部分木と比較して枝刈りを促進する
                if let Some(later) = partial_permutation.later(task_result.enumerated_permutation) {
                    partial_permutation = later;
                }

                // 未走査部分がまだ残っていたらタスク化して登録
                let Some(next_partial_permutation) = partial_permutation.next_part() else {
                    continue;
                };
                let task =
                    Self::spawn_taks(next_partial_permutation, context.clone(), callback.clone());
                tasks.push(task);

                // タスク化ずみの部分を更新
                partial_permutation = partial_permutation.next_part().unwrap();
            }
        }
    }

    fn spawn_taks<T>(
        partial_permutation: PartialPermutation,
        context: Arc<ScheduleContext>,
        mut callback: T,
    ) -> tokio::task::JoinHandle<TaskResult>
    where
        T: IParallelTreeCallback + Clone + Send + 'static,
    {
        tokio::spawn(async move {
            let decorator = TreeTraverser::default();
            let decorator = BandScheduleTraverseDecorator::new(decorator);
            let decorator = MemberConflictTraverseDecorator::new(decorator);

            // 最初のひと回しを特殊処理
            let mut partial_permutation = partial_permutation;
            {
                let current = partial_permutation.current();
                let data: Vec<i32> = current.iter().map(|x| *x as i32).collect();
                let operation = decorator.invoke(&data, context.room_assign(), context.live_info());
                match operation {
                    crate::algorithm::TraverseOperation::Next => {
                        // 候補
                        callback.notify(current);
                    }
                    crate::algorithm::TraverseOperation::Skip(index) => {
                        partial_permutation = partial_permutation.skip(index + 1);
                    }
                    crate::algorithm::TraverseOperation::Pruning => panic!("deprecated"),
                }
            }

            while let Some(permutation) = partial_permutation.next() {
                let data: Vec<i32> = permutation.current().iter().map(|x| *x as i32).collect();
                let operation = decorator.invoke(&data, context.room_assign(), context.live_info());
                match operation {
                    crate::algorithm::TraverseOperation::Next => {
                        // 候補
                        callback.notify(permutation.current());
                        partial_permutation = permutation;
                    }
                    crate::algorithm::TraverseOperation::Skip(index) => {
                        partial_permutation = permutation.skip(index + 1);
                    }
                    crate::algorithm::TraverseOperation::Pruning => panic!("deprecated"),
                };
            }

            TaskResult {
                enumerated_permutation: partial_permutation,
            }
        })
    }
}

struct TaskResult {
    pub enumerated_permutation: PartialPermutation,
}

struct ScheduleContext {
    live_info: Arc<LiveInfo>,
    room_assign: Vec<Range<usize>>,
}

impl ScheduleContext {
    pub fn live_info(&self) -> &LiveInfo {
        &self.live_info
    }

    pub fn room_assign(&self) -> &[Range<usize>] {
        &self.room_assign
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn simple() {}
}
