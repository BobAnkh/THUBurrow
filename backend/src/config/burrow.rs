use lazy_static::lazy_static;

use super::BACKEND_TEST_MODE;

pub static BURROW_PER_PAGE: usize = 10;
pub static BURROW_LIMIT: usize = 5;

lazy_static! {
    pub static ref BURROW_CREATE_DURATION: i64 = {
        if *BACKEND_TEST_MODE {
            2
        } else {
            60 * 60 * 24
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burrow_create_duration_config() {
        assert_eq!(2, *BURROW_CREATE_DURATION);
    }
}
