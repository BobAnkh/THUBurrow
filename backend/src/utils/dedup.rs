pub fn remove_duplicate(tag: Vec<String>) -> Vec<String> {
    tag.sort_unstable();
    tag.dedup();
    tag
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
