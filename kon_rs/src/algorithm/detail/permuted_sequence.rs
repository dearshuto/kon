use std::ops::{Range, RangeBounds};

use num::NumCast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermutedSequence<T, const N: usize> {
    t: [T; N],
}

impl<T, const N: usize> PermutedSequence<T, N>
where
    T: num::Integer + NumCast + Copy,
{
    pub fn new() -> Self {
        PermutedSequence {
            t: std::array::from_fn(|i| T::from(i).unwrap()),
        }
    }

    pub fn part<const M: usize>(&self, start: usize) -> Option<PermutedSequence<T, M>> {
        if start + M > N {
            return None;
        }

        Some(PermutedSequence::<T, M> {
            t: std::array::from_fn(|i| self.t[start + i]),
        })
    }
}

impl<T, const N: usize> AsRef<[T]> for PermutedSequence<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.t
    }
}

#[derive(Debug)]
pub struct PermutedSequenceIterator<T, const N: usize> {
    current: PermutedSequence<T, N>,
    range: Range<usize>,
    is_done: bool,
}

impl<T, const N: usize> PermutedSequenceIterator<T, N> {
    pub fn new(begin: PermutedSequence<T, N>) -> Self {
        Self::new_range(begin, 0..N).unwrap()
    }

    pub fn new_range<R>(current: PermutedSequence<T, N>, range: R) -> Option<Self>
    where
        R: RangeBounds<usize>,
    {
        let end = match range.end_bound() {
            std::ops::Bound::Included(index) => index + 1,
            std::ops::Bound::Excluded(index) => *index,
            std::ops::Bound::Unbounded => N,
        };

        if N < end {
            return None;
        }

        let begin = match range.start_bound() {
            std::ops::Bound::Included(index) => *index,
            std::ops::Bound::Excluded(index) => index + 1,
            std::ops::Bound::Unbounded => 0,
        };

        Some(Self {
            current,
            range: begin..end,
            is_done: false,
        })
    }
}

impl<T, const N: usize> Iterator for PermutedSequenceIterator<T, N>
where
    T: Clone + PartialOrd,
{
    type Item = PermutedSequence<T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.clone();

        let Ok(_) = super::util::next(&mut self.current.t[self.range.clone()]) else {
            if self.is_done {
                return None;
            } else {
                self.is_done = true;
                return Some(current);
            }
        };

        Some(current)
    }
}

#[derive(Debug)]
pub struct BypassPermutedSequenceIterator<T, const N: usize> {
    current: PermutedSequence<T, N>,
    depth: usize,
    is_done: bool,
}

impl<T, const N: usize> BypassPermutedSequenceIterator<T, N>
where
    T: num::Integer + num::NumCast + Copy + Clone + PartialOrd,
{
    // 指定した深度より深いノードは走査しない
    pub fn new(depth: usize) -> Self {
        Self {
            current: PermutedSequence::new(),
            depth,
            is_done: false,
        }
    }
}

impl<T, const N: usize> Iterator for BypassPermutedSequenceIterator<T, N>
where
    T: Clone + PartialOrd,
{
    type Item = PermutedSequence<T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let current = self.current.clone();

        // 逆順にして次に進めると新たな木に突入する
        self.current.t[self.depth..].reverse();
        if super::util::next(&mut self.current.t).is_err() {
            // すでに全て走査仕切っていても最初は値を返すのでフラグを立てるだけ
            self.is_done = true;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::detail::permuted_sequence::{
        BypassPermutedSequenceIterator, PermutedSequenceIterator,
    };

    use super::PermutedSequence;

    #[test]
    fn simple() {
        let x = PermutedSequence::<u8, 8>::new();
        assert_eq!(x.as_ref(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn part() {
        let x = PermutedSequence::<u8, 8>::new();

        assert_eq!(x.part::<4>(0).unwrap().as_ref(), &[0, 1, 2, 3]);
        assert_eq!(x.part::<4>(1).unwrap().as_ref(), &[1, 2, 3, 4]);
        assert_eq!(x.part::<4>(2).unwrap().as_ref(), &[2, 3, 4, 5]);
        assert_eq!(x.part::<4>(3).unwrap().as_ref(), &[3, 4, 5, 6]);
        assert_eq!(x.part::<4>(4).unwrap().as_ref(), &[4, 5, 6, 7]);
    }

    #[test]
    fn invalid_part() {
        let x = PermutedSequence::<u8, 8>::new();

        assert_eq!(x.part::<4>(5), None);
        assert_eq!(x.part::<4>(6), None);
        assert_eq!(x.part::<4>(7), None);
        assert_eq!(x.part::<4>(8), None);
        assert_eq!(x.part::<4>(9), None);
    }

    #[test]
    fn iterate() {
        let x = PermutedSequence::<u8, 3>::new();
        let mut iterator = PermutedSequenceIterator::new(x);

        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 2]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 2, 1]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[1, 0, 2]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[1, 2, 0]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[2, 0, 1]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[2, 1, 0]);
        assert_eq!(iterator.next(), None);

        // 2 回目以降も None が返ることのテスト
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_range() {
        let x = PermutedSequence::<u8, 3>::new();

        // 末尾 2 桁だけ走査
        let mut iterator = PermutedSequenceIterator::new_range(x, 1..=2).unwrap();

        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 2]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 2, 1]);
        assert_eq!(iterator.next(), None);

        // 2 回目以降も None が返ることのテスト
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_range_partial() {
        let x = PermutedSequence::<u8, 8>::new();

        // 末尾 [2, 4] で走査
        let mut iterator = PermutedSequenceIterator::new_range(x, 2..=4).unwrap();

        //                                                   |ここだけ|
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 2, 4, 3, 5, 6, 7]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 3, 2, 4, 5, 6, 7]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 3, 4, 2, 5, 6, 7]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 4, 2, 3, 5, 6, 7]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 4, 3, 2, 5, 6, 7]);
        assert_eq!(iterator.next(), None);

        // 2 回目以降も None が返ることのテスト
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_bypass() {
        // 4 桁の順列を上位 2 列分だけ列挙
        let mut iterator = BypassPermutedSequenceIterator::<u8, 4>::new(2);

        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 1, 2, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 2, 1, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[0, 3, 1, 2]);

        assert_eq!(iterator.next().unwrap().as_ref(), &[1, 0, 2, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[1, 2, 0, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[1, 3, 0, 2]);

        assert_eq!(iterator.next().unwrap().as_ref(), &[2, 0, 1, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[2, 1, 0, 3]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[2, 3, 0, 1]);

        assert_eq!(iterator.next().unwrap().as_ref(), &[3, 0, 1, 2]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[3, 1, 0, 2]);
        assert_eq!(iterator.next().unwrap().as_ref(), &[3, 2, 0, 1]);
        assert_eq!(iterator.next(), None);

        // 2 回目以降も None が返ることのテスト
        assert_eq!(iterator.next(), None);
    }
}
