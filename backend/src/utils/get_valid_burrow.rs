pub async fn get_burrow_list(burrows: String) -> Result<Vec<i64>, String> {
    let mut vec_str: Vec<&str> = burrows.split(',').collect();
    match vec_str.pop() {
        Some(_) => Ok(vec_str.iter().map(|x| x.parse::<i64>().unwrap()).collect()),
        None => Ok(Vec::new()),
    }
}
