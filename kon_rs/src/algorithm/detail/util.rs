use num::NumCast;

pub fn factional<T>(value: T) -> T
where
    T: num::Unsigned + num::Integer + NumCast + Copy,
{
    if value.is_zero() {
        NumCast::from(1).unwrap()
    } else {
        let previous = value.sub(NumCast::from(1).unwrap());
        let previous_value = factional(previous);
        value.mul(previous_value)
    }
}

pub fn next<T>(data: &mut [T]) -> Result<(), ()>
where
    T: PartialOrd,
{
    let last = data.len() - 1;
    let mut pivot = last - 1;

    // 逆順にソート済みになってない場所を見つけて
    while data[pivot] > data[pivot + 1] {
        if pivot <= 0 {
            // 樹形図の末端まで到達していた
            return Err(());
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
    let swap_pivot = pivot + 1;
    data[swap_pivot..=last].reverse();

    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn next() {
        let mut data = [0, 1, 2, 3];

        super::next(&mut data).unwrap();
        assert_eq!(data, [0, 1, 3, 2]);

        super::next(&mut data).unwrap();
        assert_eq!(data, [0, 2, 1, 3]);
    }
}
