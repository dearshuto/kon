use super::PartialPermutation;

#[allow(unused)]
pub struct PermutationTraverser {
    partial_permutation: PartialPermutation<u32>,
    is_first: bool,
}

impl PermutationTraverser {
    #[allow(unused)]
    pub fn new(digit: usize, sub_tree_depth: usize) -> Self {
        Self {
            partial_permutation: PartialPermutation::new_with(digit, digit - sub_tree_depth),
            is_first: true,
        }
    }

    #[allow(unused)]
    pub fn allocate(&mut self) -> Option<SubTree> {
        if self.is_first {
            self.is_first = false;
            return Some(SubTree {
                partial_permutation: self.partial_permutation.clone(),
                is_first: true,
            });
        }

        let Some(next_part) = self.partial_permutation.next_part() else {
            return None;
        };

        self.partial_permutation = next_part.clone();

        Some(SubTree {
            partial_permutation: next_part.clone(),
            is_first: true,
        })
    }
}

pub struct SubTree {
    partial_permutation: PartialPermutation<u32>,
    is_first: bool,
}

impl Iterator for SubTree {
    type Item = PartialPermutation<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_first {
            self.is_first = false;
            return Some(self.partial_permutation.clone());
        }

        let Some(next_permutation) = self.partial_permutation.next() else {
            return None;
        };

        self.partial_permutation = next_permutation;
        Some(self.partial_permutation.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::PermutationTraverser;

    #[test]
    fn simple() {
        // [0]
        let mut traverser = PermutationTraverser::new(1, 1);
        let mut sub_tree = traverser.allocate().unwrap();
        assert_eq!(sub_tree.next().unwrap().current(), [0]);
    }

    #[test]
    fn single_tree() {
        // [0, 1, 2]
        let mut traverser = PermutationTraverser::new(3, 3);
        let mut sub_tree = traverser.allocate().unwrap();
        assert_eq!(sub_tree.next().unwrap().current(), [0, 1, 2]);
        assert_eq!(sub_tree.next().unwrap().current(), [0, 2, 1]);
        assert_eq!(sub_tree.next().unwrap().current(), [1, 0, 2]);
        assert_eq!(sub_tree.next().unwrap().current(), [1, 2, 0]);
        assert_eq!(sub_tree.next().unwrap().current(), [2, 0, 1]);
        assert_eq!(sub_tree.next().unwrap().current(), [2, 1, 0]);

        assert!(traverser.allocate().is_none());
    }

    #[test]
    fn multi_tree() {
        // 3 桁の順列を下 2 桁で区切った以下 3 つの走査
        // [0, 1, 2]
        // [1, 0, 2]
        // [2, 0, 1]
        let mut traverser = PermutationTraverser::new(3, 2);
        let mut sub_tree_0 = traverser.allocate().unwrap();
        assert_eq!(sub_tree_0.next().unwrap().current(), [0, 1, 2]);
        assert_eq!(sub_tree_0.next().unwrap().current(), [0, 2, 1]);

        let mut sub_tree_1 = traverser.allocate().unwrap();
        assert_eq!(sub_tree_1.next().unwrap().current(), [1, 0, 2]);
        assert_eq!(sub_tree_1.next().unwrap().current(), [1, 2, 0]);

        let mut sub_tree_2 = traverser.allocate().unwrap();
        assert_eq!(sub_tree_2.next().unwrap().current(), [2, 0, 1]);
        assert_eq!(sub_tree_2.next().unwrap().current(), [2, 1, 0]);

        assert!(traverser.allocate().is_none());
    }
}
