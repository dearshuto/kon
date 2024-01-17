// 樹形図を走査するときに各パターンで呼び出されるコールバック
// インデックスを返すとそれ以下のツリーの走査をスキップします。
// 例: [0, 2, 1, 3] で 0 を返すと次は [1, 0, 2, 3] でコールバックが呼び出されます。
pub trait ITreeCallback {
    fn invoke(&mut self, indicies: &[i32]) -> Option<usize>;
}

// 関数オブジェクトを利用するためのアダプター
struct TreeCallback<TFunc: FnMut(&[i32])> {
    func: TFunc,
}
impl<TFunc: FnMut(&[i32])> ITreeCallback for TreeCallback<TFunc> {
    fn invoke(&mut self, indicies: &[i32]) -> Option<usize> {
        (self.func)(indicies);
        None
    }
}

// 関数オブジェクトで注入するパターン
#[allow(dead_code)]
pub fn traverse_all_with_callback<TFunc: FnMut(&[i32])>(data: &mut Vec<i32>, callback: TFunc) {
    let mut tree_callback = TreeCallback { func: callback };
    traverse_all::<TreeCallback<TFunc>>(data, &mut tree_callback);
}

// より詳細な実装を注入するパターン
#[allow(dead_code)]
pub fn traverse_all<T>(data: &mut Vec<i32>, callback: &mut T)
where
    T: ITreeCallback,
{
    loop {
        // コールバックでインデックスを指定されたらツリーの走査をスキップ
        if let Some(skip_index) = callback.invoke(data) {
            let mut sort_buffer = data.split_off(skip_index + 1);
            sort_buffer.sort();
            sort_buffer.reverse();
            data.append(&mut sort_buffer);
        }

        let mut last = data.len() - 1;
        let mut pivot = last - 1;

        // 逆順にソート済みになってない場所を見つけて
        while data[pivot] > data[pivot + 1] {
            if pivot == 0 {
                // 樹形図の末端まで到達していた
                return;
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
    }
}

#[cfg(test)]
mod tests {
    use super::ITreeCallback;

    #[test]
    fn traverse_with_callback() {
        let mut result: Vec<Vec<i32>> = Vec::default();
        super::traverse_all_with_callback(&mut vec![0, 1, 2], |data| {
            result.push(data.to_vec());
        });

        assert_eq!(result[0], vec![0, 1, 2]);
        assert_eq!(result[1], vec![0, 2, 1]);
        assert_eq!(result[2], vec![1, 0, 2]);
        assert_eq!(result[3], vec![1, 2, 0]);
        assert_eq!(result[4], vec![2, 0, 1]);
        assert_eq!(result[5], vec![2, 1, 0]);
    }

    #[derive(Default)]
    struct Traverse {
        data: Vec<Vec<i32>>,
    }

    impl ITreeCallback for Traverse {
        fn invoke(&mut self, indicies: &[i32]) -> Option<usize> {
            self.data.push(indicies.to_vec());
            None
        }
    }

    #[test]
    fn traverse_with_object() {
        let mut traverse = Traverse::default();
        super::traverse_all(&mut vec![0, 1, 2], &mut traverse);

        assert_eq!(traverse.data[0], vec![0, 1, 2]);
        assert_eq!(traverse.data[1], vec![0, 2, 1]);
        assert_eq!(traverse.data[2], vec![1, 0, 2]);
        assert_eq!(traverse.data[3], vec![1, 2, 0]);
        assert_eq!(traverse.data[4], vec![2, 0, 1]);
        assert_eq!(traverse.data[5], vec![2, 1, 0]);
    }

    // 樹形図の一部だけを走査するテスト
    // 樹形図のルート直下をそれぞれ 1 要素だけ列挙するようにしている
    #[derive(Default)]
    struct TraversePartial {
        data: Vec<Vec<i32>>,
    }
    impl ITreeCallback for TraversePartial {
        fn invoke(&mut self, indicies: &[i32]) -> Option<usize> {
            self.data.push(indicies.to_vec());
            // ルート直下は 1 度走査したら以降はスキップする
            Some(0)
        }
    }
    #[test]
    fn traverse_partial() {
        let mut traverse = TraversePartial::default();
        super::traverse_all(&mut vec![0, 2, 1, 3], &mut traverse);

        // 先頭の要素が 0 の組み合わせは [0, 2, 3, 1] や [0, 3, 1, 2] などあるが、
        // 0 番目を指定してスキップされるので初期値の [0, 2, 1, 3] 以外の要素はスキップされる
        assert_eq!(traverse.data[0], vec![0, 2, 1, 3]);

        // 0 番目の要素が変わると 1 つだけ走査する
        assert_eq!(traverse.data[1], vec![1, 0, 2, 3]);
        assert_eq!(traverse.data[2], vec![2, 0, 1, 3]);
        assert_eq!(traverse.data[3], vec![3, 0, 1, 2]);
    }
}
