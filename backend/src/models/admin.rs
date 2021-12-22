//! Models of admin

use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum AdminOperation {
    BanUser{uid: i64},
    ReopenUser{uid: i64},
    BanBurrow{burrow_id: i64},
    ReopenBurrow{burrow_id: i64},
    BanPost{post_id: i64},
    ReopenPost{post_id: i64},
    BanReply{post_id: i64, reply_id: i32},
    ReopenReply{post_id: i64, reply_id: i32},
    CreateAdmin{uid: i64},
    DeleteAdmin{uid: i64},
    SetAdminRole{uid: i64, role: i32},
    GetUserId{burrow_id: i64},
}
