use lazy_static::lazy_static;

use super::BACKEND_TEST_MODE;

lazy_static! {
    pub static ref MAX_IMAGE_NUM: i32 = {
        if *BACKEND_TEST_MODE {
            3
        } else {
            1000
        }
    };
}
