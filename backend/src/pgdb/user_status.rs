//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_status")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uid: i64,
    // the latest time of user creating a burrow
    pub update_time: DateTimeWithTimeZone,
    pub user_state: i32,
    #[sea_orm(column_type = "Text")]
    pub valid_burrow: String,
    #[sea_orm(column_type = "Text")]
    pub banned_burrow: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
