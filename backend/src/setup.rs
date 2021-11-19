use crate::pgdb;
use crate::pool::PgDb;
use rocket::{fairing, Build, Rocket};
use rocket_db_pools::Database;
use sea_orm::query::ConnectionTrait;
use sea_orm::sea_query::{ColumnDef, Index, PostgresQueryBuilder, SchemaStatementBuilder};
use sea_orm::{error::*, query::Statement, sea_query, DbConn, ExecResult};

async fn build_statement<T>(db: &DbConn, stmt: &T) -> Result<ExecResult, DbErr>
where
    T: SchemaStatementBuilder,
{
    let builder = db.get_database_backend();
    db.execute(Statement::from_string(
        builder,
        stmt.build(PostgresQueryBuilder),
    ))
    .await
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
                .text()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::user::Column::Password)
                .text()
                .not_null(),
        )
        .col(ColumnDef::new(pgdb::user::Column::Email).text().not_null())
        .col(
            ColumnDef::new(pgdb::user::Column::CreateTime)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(ColumnDef::new(pgdb::user::Column::Salt).text().not_null())
        .to_owned();
    // println!("user table: {}", stmt.to_string(PostgresQueryBuilder));
    build_statement(db, &stmt).await
}

pub async fn create_user_index_username(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = Index::create()
        .name("idx-username")
        .table(pgdb::user::Entity)
        .col(pgdb::user::Column::Username)
        .to_owned();
    build_statement(db, &stmt).await
}

pub async fn create_user_index_email(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = Index::create()
        .name("idx-email")
        .table(pgdb::user::Entity)
        .col(pgdb::user::Column::Email)
        .to_owned();
    build_statement(db, &stmt).await
}

pub async fn create_image_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(pgdb::image::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(pgdb::image::Column::Filename)
                .text()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(pgdb::image::Column::UserId)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::image::Column::Size)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::image::Column::CreateTime)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::image::Column::LastDownloadTime)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned();
    // println!("image table: {}", stmt.to_string(PostgresQueryBuilder));
    build_statement(db, &stmt).await
}

pub async fn create_user_status_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(pgdb::user_status::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(pgdb::user_status::Column::Uid)
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(pgdb::user_status::Column::ModifiedTime)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(
            ColumnDef::new(pgdb::user_status::Column::Banned)
                .small_integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(pgdb::user_status::Column::ValidBurrow)
                .text()
                .not_null()
                .default("-1".to_string()),
        )
        .to_owned();
    build_statement(db, &stmt).await
}

// pub async fn alter_image_table(db: &DbConn) -> Result<ExecResult, DbErr> {
//     let stmt = sea_query::Table::alter()
//         .table(pgdb::image::Entity)
//         .add_column(
//             ColumnDef::new(pgdb::image::Column::Size)
//                 .integer()
//                 .not_null(),
//         )
//         .to_owned();
//     println!("image table: {}", stmt.to_string(PostgresQueryBuilder));
//     build_statement(db, &stmt).await
// }

pub async fn user_table_setup(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = create_user_table(conn).await;
    let _ = create_image_table(conn).await;
    let _ = create_user_index_username(conn).await;
    let _ = create_user_index_email(conn).await;
    let _ = create_user_status_table(conn).await;
    // let _ = alter_image_table(conn).await;
    Ok(rocket)
}

pub async fn create_burrow_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
    .table(pgdb::burrow::Entity)
    .if_not_exists()
    .col(
        ColumnDef::new(pgdb::burrow::Column::Id)
            .big_integer()
            .not_null()
            .primary_key()
            .auto_increment(),
    )
    .col(
        ColumnDef::new(pgdb::burrow::Column::Title)
            .string()
            .not_null(),
    )
    .col(
        ColumnDef::new(pgdb::burrow::Column::Description)
            .string(),
    )
    .col(
        ColumnDef::new(pgdb::burrow::Column::Author)
            .big_integer()
            .not_null(),
    )
    .col(
        ColumnDef::new(pgdb::burrow::Column::Status)
            .small_integer()
            .not_null()
            .default(0),
    )
    .to_owned();
    build_statement(db, &stmt).await
}

// pub async fn alter_burrow_table(db: &DbConn) -> Result<ExecResult, DbErr> {
//     let stmt = sea_query::Table::alter()
//     .table(pgdb::burrow::Entity)
//     .add_column(
//         ColumnDef::new(types::Alias::new("title"))
//             .text()
//             .not_null()
//             .default("this is a title"),
//     )
//     .to_owned();
//     println!("burrow table: {}", stmt.to_string(PostgresQueryBuilder));

//     build_statement(db, &stmt).await
// }

pub async fn burrow_table_setup(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = create_burrow_table(conn).await;
    // let _ = alter_burrow_table(conn).await;
    Ok(rocket)
}

// pub async fn create_content_table(db: &DbConn) -> Result<ExecResult, DbErr> {
//     let stmt = sea_query::Table::create()
//     .table(pgdb::burrow::Entity)
//     .if_not_exists()
//     .col(
//         ColumnDef::new(pgdb::burrow::Column::PostId)
//             .big_integer()
//             .not_null()
//             .primary_key()
//             .auto_increment(),
//     )
//     .col(
//         ColumnDef::new(pgdb::burrow::Column::Title)
//             .string()
//             .not_null(),
//     )
//     .col(
//         ColumnDef::new(pgdb::burrow::Column::Description)
//             .string(),
//     )
//     .col(
//         ColumnDef::new(pgdb::burrow::Column::Author)
//             .big_integer()
//             .not_null(),
//     )
//     .col(
//         ColumnDef::new(pgdb::burrow::Column::Status)
//             .small_integer()
//             .not_null()
//             .default(0),
//     )
//     .to_owned();
//     build_statement(db, &stmt).await
// }

pub async fn create_junction_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
    .table(pgdb::favorite::Entity)
    .if_not_exists()
    .col(
        ColumnDef::new(pgdb::favorite::Column::Userid)
            .big_integer()
            .not_null(),
    )
    .col(
        ColumnDef::new(pgdb::favorite::Column::Burrowid)
            .big_integer()
            .not_null(),
    )
    .primary_key(Index::create()
        .col(pgdb::favorite::Column::Userid)
        .col(pgdb::favorite::Column::Burrowid)
    )
    .to_owned();
    build_statement(db, &stmt).await
}

pub async fn junction_table_setup(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = create_junction_table(conn).await;
    Ok(rocket)
}
