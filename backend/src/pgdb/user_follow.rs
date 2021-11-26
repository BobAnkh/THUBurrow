use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_follow")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: i64,
    #[sea_orm(primary_key)]
    pub burrow_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Uid",
        to = "super::user::Column::Uid"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::burrow::Entity",
        from = "Column::BurrowId",
        to = "super::burrow::Column::BurrowId"
    )]
    Burrow,
}

impl ActiveModelBehavior for ActiveModel {}
