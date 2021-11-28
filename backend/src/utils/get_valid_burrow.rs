pub fn get_burrow_list(burrows: &str) -> Vec<i64> {
    if burrows.is_empty() {
        return Vec::new();
    }
    burrows
        .split(',')
        .map(|x| x.parse::<i64>().unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_burrow_list() {
        let v1: Vec<i64> = Vec::new();
        assert_eq!(get_burrow_list(""), v1);
        assert_eq!(get_burrow_list("1"), vec![1]);
        assert_eq!(get_burrow_list("1,2"), vec![1, 2]);
    }
}
