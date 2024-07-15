use num::NumCast;

#[allow(unused)]
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
