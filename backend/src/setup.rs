use crate::pgdb;
use crate::pool::PgDb;
use rocket::{fairing, Build, Rocket};
use rocket_db_pools::Database;
use sea_orm::query::ConnectionTrait;
use sea_orm::sea_query::{ColumnDef, TableCreateStatement};
use sea_orm::{error::*, sea_query, DbConn, ExecResult};

async fn create_table(db: &DbConn, stmt: &TableCreateStatement) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    db.execute(builder.build(stmt)).await
}

pub async fn create_user_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(pgdb::user::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(pgdb::user::Column::Uid)
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(pgdb::user::Column::Username)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::user::Column::Password)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::user::Column::Email)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(pgdb::user::Column::Salt).string().not_null())
        .to_owned();

    create_table(db, &stmt).await
}

pub async fn user_table_setup(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = create_user_table(conn).await;
    Ok(rocket)
}
