pub fn remove_tag_duplicate(tag: &Vec<String>) -> Vec<String> {
    let mut tag_check = tag.to_owned();
    tag_check.sort_unstable();
    tag_check.dedup();
    tag_check
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_tag_duplicate() {
        let tag = vec!["NoTag".to_string(), "HaveTag".to_string()];
        let tag_dup = vec!["NoTag".to_string(), "NoTag".to_string()];
        let tag_emp: Vec<String> = vec![];
        assert_eq!(remove_tag_duplicate(&tag), ["HaveTag", "NoTag"]);
        assert_eq!(remove_tag_duplicate(&tag_dup), ["NoTag"]);
        assert_eq!(remove_tag_duplicate(&tag_emp), tag_emp);
    }
}
