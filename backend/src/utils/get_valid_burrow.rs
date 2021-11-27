pub fn get_burrow_list(burrows: String) -> Vec<i64> {
    if burrows.is_empty() {
        return Vec::new();
    }
    burrows.split(',').map(|x| x.parse::<i64>().unwrap()).collect()
}
