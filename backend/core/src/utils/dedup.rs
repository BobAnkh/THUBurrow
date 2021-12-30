/// Remove duplicate elements from a vector.
///
/// Use sort_unstable and dedup to remove duplicates from a vector.
///
/// ## Parameters
///
/// - `v`: The vector to dedup.
///
/// ## Generic Parameters
///
/// - `T`: The type of the elements in the vector. Should be `Ord` and `PartialEq`.
///
/// ## Returns
///
/// A vector with no duplicate elements.
pub fn remove_duplicate<T: Ord + PartialEq>(mut v: Vec<T>) -> Vec<T> {
    v.sort_unstable();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicate() {
        let tag = vec!["NoTag".to_string(), "HaveTag".to_string()];
        let tag_dup = vec!["NoTag".to_string(), "NoTag".to_string()];
        let tag_emp: Vec<String> = Vec::new();
        let tag_check: Vec<String> = Vec::new();
        assert_eq!(remove_duplicate(tag), ["HaveTag", "NoTag"]);
        assert_eq!(remove_duplicate(tag_dup), ["NoTag"]);
        assert_eq!(remove_duplicate(tag_emp), tag_check);
    }
}
