use itertools::Itertools;
use num::NumCast;

use super::util;

#[derive(Debug, Clone)]
pub struct PartialPermutation<T: num::Integer + NumCast + Clone + Copy> {
    data: Vec<T>,
    start: usize,
}

impl PartialPermutation<u8> {
    /// 順列生成器を生成します。
    pub fn new(digit: usize, start: usize) -> Self {
        Self::new_with(digit, start)
    }
}

impl<T: num::Integer + NumCast + Clone + Copy> PartialPermutation<T> {
    /// 順列生成器を生成します。
    pub fn new_with(digit: usize, start: usize) -> Self {
        let data = (0..digit)
            .map(|x| NumCast::from(x).unwrap())
            .collect::<Vec<T>>();

        Self {
            data,
            start: digit.min(start),
        }
    }

    /// 順列生成器が生成可能な最後の順列を生成します。
    pub fn last(&self) -> Self {
        let data_count = self.data.len();

        // 先頭部分を vec 化
        let mut data = self.data[0..self.start].to_vec();

        (0..self.start).for_each(|index| data[index] = self.data[index]);

        // 走査部分の最後の並びをコピー。走査部分を逆順にすればよい
        let tail_iterator = (self.start..data_count)
            .map(|index| self.data[index])
            .sorted()
            .rev();
        data.extend(tail_iterator);

        Self {
            data,
            start: self.start,
        }
    }

    /// 順列生成器の次の木構造を生成します。
    pub fn next_part(&self) -> Option<Self> {
        // 現在の部分の最後の並びを取得して、それを一つ進めれば次の部分となる
        let mut last = self.last();
        last.start = 0;

        let mut next_part = last.next()?;
        next_part.start = self.start;
        Some(next_part)
    }

    pub fn next(&self) -> Option<PartialPermutation<T>> {
        // 長さ 1 のときはつぎの順列は存在しないので常に None でよい
        if self.data.len() == 1 {
            return None;
        }

        let mut data = self.data.clone();
        let mut last = data.len() - 1;
        let mut pivot = last - 1;

        // 逆順にソート済みになってない場所を見つけて
        while data[pivot] > data[pivot + 1] {
            if pivot <= self.start {
                // 樹形図の末端まで到達していた
                return None;
            }
            pivot -= 1;
        }

        // 値を入れ替えて
        let mut second = last;
        while data[pivot] > data[second] {
            second -= 1;
        }
        data.swap(pivot, second);

        // 値を入れ替えた場所以降は逆順にソート済みなので reverse すると新たな木に突入する
        // reverse
        let mut swap_pivot = pivot + 1;
        while swap_pivot < last {
            data.swap(swap_pivot, last);
            swap_pivot += 1;
            last -= 1;
        }

        Some(Self {
            data,
            start: self.start,
        })
    }

    pub fn current(&self) -> &[T] {
        &self.data
    }

    pub fn skip(&self, index: usize) -> Self {
        let index = index.min(self.data.len());
        let mut new_data = self.data.clone();
        new_data[index..].sort();
        new_data[index..].reverse();

        Self {
            data: new_data,
            start: self.start,
        }
    }

    /// 順列を比較して辞書的に後になる方を返します
    pub fn later(&self, other: Self) -> Option<Self> {
        if self.data.len() != other.data.len() {
            panic!();
        }

        for index in 0..self.data.len() {
            if self.data[index] < other.data[index] {
                return Some(other);
            } else {
                return None;
            }
        }

        None
    }

    /// 部分順列の数を算出します
    pub fn calculate_count(&self) -> usize {
        let length = self.data.len() - self.start;
        util::factional(length)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::PartialPermutation;

    #[test]
    fn simple() {
        let mut v = vec![0];
        v.swap_remove(0);

        // 4 ケタ全範囲（4! と等価）
        let mut partial_permutation = PartialPermutation::new(4, 0);
        let mut result = vec![partial_permutation.current().to_vec()];
        while let Some(permutation) = partial_permutation.next() {
            result.push(permutation.current().to_vec());
            partial_permutation = permutation;
        }

        // 全要素列挙できているか外部クレートを使って比較
        let permutations = (0..4).permutations(4);
        itertools::assert_equal(permutations, result);
    }

    #[test]
    fn simple_3() {
        // 3 ケタ全範囲（3! と等価）
        let p0 = PartialPermutation::new(3, 0);
        assert!(p0.current() == [0, 1, 2]);

        let Some(p1) = p0.next() else {
            panic!();
        };
        assert!(p1.current() == [0, 2, 1]);

        let Some(p2) = p1.next() else {
            panic!();
        };
        assert!(p2.current() == [1, 0, 2]);

        let Some(p3) = p2.next() else {
            panic!();
        };
        assert!(p3.current() == [1, 2, 0]);

        let Some(p4) = p3.next() else {
            panic!();
        };
        assert!(p4.current() == [2, 0, 1]);

        let Some(p5) = p4.next() else {
            panic!();
        };
        assert!(p5.current() == [2, 1, 0]);
    }

    #[test]
    fn last() {
        // 4 ケタ末尾 3 ケタだけ
        let partial_permutation = PartialPermutation::new(4, 1);

        // 0321 を期待
        let last = partial_permutation.last();
        itertools::assert_equal(last.data, vec![0u8, 3, 2, 1]);
    }

    #[test]
    fn next_nothing() {
        // 4 ケタ全範囲（4! と等価）
        let partial_permutation = PartialPermutation::new(1, 0);
        let next = partial_permutation.next();
        assert!(next.is_none());
    }

    #[test]
    fn next_part() {
        // 4 ケタ末尾 3 ケタだけ
        // 0123 -> 0132 -> ... -> 0321
        let partial_permutation = PartialPermutation::new(4, 1);

        let next_part = partial_permutation.next_part().unwrap();
        itertools::assert_equal(next_part.data, vec![1u8, 0, 2, 3]);
    }

    #[test]
    fn next_part_not_found() {
        // 4 ケタ末尾 3 ケタだけ
        // 0123~
        let partial_permutation = PartialPermutation::new(4, 1);

        // 1023〜
        let next_part = partial_permutation.next_part().unwrap();

        // 2013〜
        let next_part = next_part.next_part().unwrap();

        // 3012〜
        let next_part = next_part.next_part().unwrap();

        // つぎはもうない
        assert!(next_part.next_part().is_none());
    }

    #[test]
    fn later() {
        // 4 ケタ末尾 3 ケタだけ
        // 0123~
        let partial_permutation = PartialPermutation::new(4, 1);

        // 1023〜
        let next_part = partial_permutation.next_part().unwrap();

        // 1023~ で更新は起きないはず
        assert!(next_part.later(partial_permutation).is_none());
        itertools::assert_equal(&vec![1, 0, 2, 3], next_part.current());
    }

    #[test]
    fn later_changed() {
        // 4 ケタ末尾 3 ケタだけ
        // 0123~
        let partial_permutation = PartialPermutation::new(4, 1);

        // 1023〜
        let next_part = partial_permutation.next_part().unwrap();

        // 1023~ が返るはず
        let Some(later) = partial_permutation.later(next_part) else {
            panic!();
        };
        itertools::assert_equal(&vec![1, 0, 2, 3], later.current());
    }

    #[test]
    fn concat_parallel() {
        // 4 ケタ末尾 3 ケタだけ
        let mut partial_permutation = PartialPermutation::new(4, 1);

        let mut result = vec![partial_permutation.current().to_vec()];
        loop {
            // 部分の順列をすべて走査
            while let Some(permutation) = partial_permutation.next() {
                result.push(permutation.current().to_vec());

                partial_permutation = permutation;
            }

            // さらにその次があったらループを続ける
            let Some(next_partial) = partial_permutation.next_part() else {
                break;
            };

            result.push(next_partial.current().to_vec());
            partial_permutation = next_partial;
        }

        // 4! と等価なことを確認
        let permutations = (0..4).permutations(4);
        itertools::assert_equal(permutations, result);
    }

    #[test]
    fn skip() {
        // 4 ケタ末尾 3 ケタだけ
        let partial_permutation = PartialPermutation::new(4, 1);

        // 最初の並び
        itertools::assert_equal(&vec![0, 1, 2, 3], partial_permutation.current());

        // 左から 1 番目以降が走査ずみに
        let skip_partial_permutation = partial_permutation.skip(1);
        itertools::assert_equal(&vec![0, 3, 2, 1], skip_partial_permutation.current());

        // 全て走査済みに
        let skip_partial_permutation = skip_partial_permutation.skip(0);
        itertools::assert_equal(&vec![3, 2, 1, 0], skip_partial_permutation.current());
        // スキップ後の従列がケタから外れているのでもう操作できない
        assert!(skip_partial_permutation.next().is_none());
    }
}
