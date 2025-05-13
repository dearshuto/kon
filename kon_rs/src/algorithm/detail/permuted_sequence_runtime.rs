use num::{Integer, NumCast};

use super::permuted_sequence::{BypassPermutedSequenceIterator, PermutedSequenceIterator};

pub struct PermutedSequenceRuntime<T, const N: usize, const JOBS: usize> {
    senders: [tokio::sync::mpsc::Sender<PermutedSequenceIterator<T, N>>; JOBS],
}

impl<T, const N: usize, const JOBS: usize> PermutedSequenceRuntime<T, N, JOBS> {
    pub fn new() -> (
        Self,
        [tokio::sync::mpsc::Receiver<PermutedSequenceIterator<T, N>>; JOBS],
    ) {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let mut senders: [tokio::sync::mpsc::Sender<PermutedSequenceIterator<T, N>>; JOBS] =
            std::array::from_fn(|_| sender.clone());
        let receivers = std::array::from_fn(|index| {
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            senders[index] = sender;
            receiver
        });

        (Self { senders }, receivers)
    }
}

impl<T, const N: usize, const JOBS: usize> PermutedSequenceRuntime<T, N, JOBS>
where
    T: Integer + NumCast + Copy + PartialOrd,
{
    pub async fn serve(self, depth: usize) {
        for (index, permuted_sequence) in BypassPermutedSequenceIterator::new(depth).enumerate() {
            let iterator = PermutedSequenceIterator::new_range(permuted_sequence, depth..).unwrap();
            let index = index % JOBS;
            let sender = &self.senders[index];
            sender.send(iterator).await.unwrap_or_default();
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::PermutedSequenceRuntime;

    fn runtime(depth: usize) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (permuted_sequence_runtime, receivers) = PermutedSequenceRuntime::<u8, 4, 4>::new();

        runtime.spawn(async move { permuted_sequence_runtime.serve(depth).await });

        let x = receivers.map(|mut receiver| {
            runtime.spawn(async move {
                let mut vec = Vec::default();
                while let Some(iterator) = receiver.recv().await {
                    for permuted_sequrnce in iterator {
                        vec.push(permuted_sequrnce);
                    }
                }

                vec
            })
        });

        let receive_job = runtime.block_on(async move { futures::future::join_all(x).await });
        let mut sequence: Vec<_> = receive_job
            .into_iter()
            .filter_ok(|_x| true)
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .collect();
        sequence.sort();

        assert_eq!(sequence[0].as_ref(), &[0, 1, 2, 3]);
        assert_eq!(sequence[1].as_ref(), &[0, 1, 3, 2]);
        assert_eq!(sequence[2].as_ref(), &[0, 2, 1, 3]);
        assert_eq!(sequence[3].as_ref(), &[0, 2, 3, 1]);
        assert_eq!(sequence[4].as_ref(), &[0, 3, 1, 2]);
        assert_eq!(sequence[5].as_ref(), &[0, 3, 2, 1]);

        assert_eq!(sequence[6].as_ref(), &[1, 0, 2, 3]);
        assert_eq!(sequence[7].as_ref(), &[1, 0, 3, 2]);
        assert_eq!(sequence[8].as_ref(), &[1, 2, 0, 3]);
        assert_eq!(sequence[9].as_ref(), &[1, 2, 3, 0]);
        assert_eq!(sequence[10].as_ref(), &[1, 3, 0, 2]);
        assert_eq!(sequence[11].as_ref(), &[1, 3, 2, 0]);

        assert_eq!(sequence[12].as_ref(), &[2, 0, 1, 3]);
        assert_eq!(sequence[13].as_ref(), &[2, 0, 3, 1]);
        assert_eq!(sequence[14].as_ref(), &[2, 1, 0, 3]);
        assert_eq!(sequence[15].as_ref(), &[2, 1, 3, 0]);
        assert_eq!(sequence[16].as_ref(), &[2, 3, 0, 1]);
        assert_eq!(sequence[17].as_ref(), &[2, 3, 1, 0]);

        assert_eq!(sequence[18].as_ref(), &[3, 0, 1, 2]);
        assert_eq!(sequence[19].as_ref(), &[3, 0, 2, 1]);
        assert_eq!(sequence[20].as_ref(), &[3, 1, 0, 2]);
        assert_eq!(sequence[21].as_ref(), &[3, 1, 2, 0]);
        assert_eq!(sequence[22].as_ref(), &[3, 2, 0, 1]);
        assert_eq!(sequence[23].as_ref(), &[3, 2, 1, 0]);
    }

    #[test]
    fn depth_0() {
        runtime(0);
    }

    #[test]
    fn depth_1() {
        runtime(1);
    }

    #[test]
    fn depth_2() {
        runtime(2);
    }
}
