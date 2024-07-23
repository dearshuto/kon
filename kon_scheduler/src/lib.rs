use std::collections::HashMap;

pub fn parse_bands<T, U>(iterator: T) -> HashMap<String, Vec<String>>
where
    T: IntoIterator<Item = U>,
    U: AsRef<str>,
{
    iterator
        .into_iter()
        .map(|x| {
            let str: &str = x.as_ref();
            let mut inputs = str.split('/');
            let band_name = inputs.next().unwrap().to_string();
            let members: Vec<String> = inputs.map(|x| x.to_string()).collect();
            (band_name, members)
        })
        .collect()
}

pub fn parse_schedule<T, U>(iterator: T) -> HashMap<String, Vec<bool>>
where
    T: IntoIterator<Item = U>,
    U: AsRef<str>,
{
    iterator
        .into_iter()
        .map(|x| {
            let mut inputs = x.as_ref().split('/');
            let band_name = inputs.next().unwrap().to_string();
            let schedule: Vec<bool> = inputs
                .map(|x| if x == "true" { true } else { false })
                .collect();
            (band_name, schedule)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{parse_bands, parse_schedule};

    #[test]
    fn simple_parse_bands() {
        let expected = HashMap::from_iter([
            (
                "a".to_string(),
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ),
            ("b".to_string(), vec!["b".to_string()]),
            ("c".to_string(), vec!["c".to_string()]),
            ("d".to_string(), vec!["d".to_string()]),
        ]);

        let actual = parse_bands(["a/a/b/c", "b/b", "c/c", "d/d"]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_parse_schedule() {
        let expected = HashMap::from_iter([
            ("a".to_string(), vec![true, false, true]),
            ("b".to_string(), vec![true, false, false]),
            ("c".to_string(), vec![true, true, true]),
            ("d".to_string(), vec![true, false, true]),
        ]);

        let actual = parse_schedule([
            "a/true/false/true",
            "b/true/false/false",
            "c/true/true/true",
            "d/true/false/true",
        ]);
        assert_eq!(expected, actual);
    }
}
