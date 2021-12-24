//! Module to valid burrow
//!
//! Split burrow list stored in database to burrow list

/// Split burrow list stored in database as string to burrow_id list
///
/// ## Parameters
///
/// - `burrow`: The burrow list stored in database as string.
///
/// ## Returns
///
/// A vector of burrow_id.
pub fn get_burrow_list(burrows: &str) -> Vec<i64> {
    if burrows.is_empty() {
        return Vec::new();
    }
    burrows
        .split(',')
        .map(|x| x.parse::<i64>().unwrap())
        .collect()
}

/// Check if a given burrow_id is valid
///
/// ## Parameters
///
/// - `burrow`: The burrow list stored in database as string.
/// - `burrow_id`: The burrow_id to check.
///
/// ## Returns
///
/// A boolean value. True if the burrow_id is valid.
pub fn is_valid_burrow(burrows: &str, burrow_id: &i64) -> bool {
    if burrows.is_empty() {
        return false;
    }
    let burrow_list = get_burrow_list(burrows);
    burrow_list.contains(burrow_id)
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

    #[test]
    fn test_is_valid_burrow() {
        assert_eq!(is_valid_burrow("", &1), false);
        assert_eq!(is_valid_burrow("1", &1), true);
        assert_eq!(is_valid_burrow("1,2", &1), true);
        assert_eq!(is_valid_burrow("1,2", &2), true);
        assert_eq!(is_valid_burrow("1,2", &3), false);
    }
}
