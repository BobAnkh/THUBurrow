use lazy_static::lazy_static;

use super::BACKEND_TEST_MODE;

pub static POST_PER_PAGE: usize = 20;
pub static REPLY_PER_PAGE: usize = 20;
pub static MAX_SECTION: usize = 3;
pub static MAX_TAG: usize = 10;

lazy_static! {
    pub static ref POST_DELETE_DURATION: i64 = {
        if *BACKEND_TEST_MODE {
            2
        } else {
            60 * 2
        }
    };
}
