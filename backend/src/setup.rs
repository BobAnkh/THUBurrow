//! Setup of PostgreSQL database.

/// Initialize the unique id generator.
pub mod id_generator {
    use idgenerator::{IdGeneratorOptions, IdHelper};

    pub fn init(worker_id: u32) {
        IdHelper::init();
        let mut options: IdGeneratorOptions = IdGeneratorOptions::new(worker_id);
        options.worker_id_bit_len = 6;
        options.seq_bit_len = 16;
        IdHelper::set_id_generator(options);
    }
}

/// Initialize the CORS handler.
pub mod cors {
    use rocket_cors::{AllowedOrigins, Cors};

    pub fn init() -> Cors {
        let allowed_origins = AllowedOrigins::some(
            &["https://thuburrow.com"],
            &["^https://(.+).thuburrow.com$"],
        );

        // You can also deserialize this
        let cors = rocket_cors::CorsOptions {
            allowed_origins,
            allow_credentials: true,
            max_age: Some(3600),
            ..Default::default()
        }
        .to_cors();
        match cors {
            Ok(c) => c,
            _ => panic!("Can not initialize cors"),
        }
    }
}

/// Initialize the PostgreSQL database.
pub mod postgres {
    use rocket::{fairing, Build, Rocket};
    use rocket_db_pools::Database;
    use sea_orm::query::{ConnectionTrait, Statement};
    use sea_orm::sea_query::{
        self, ColumnDef, Index, PostgresQueryBuilder, SchemaStatementBuilder,
    };
    use sea_orm::{error::*, DbConn, ExecResult};

    use crate::db;
    use crate::pool::PgDb;

    pub async fn postgres_table_setup(rocket: Rocket<Build>) -> fairing::Result {
        let conn = &PgDb::fetch(&rocket).unwrap().connection;
        let _ = create_user_table(conn).await;
        let _ = create_image_table(conn).await;
        let _ = create_user_index_username(conn).await;
        let _ = create_user_index_email(conn).await;
        let _ = create_content_post_table(conn).await;
        let _ = create_content_reply_table(conn).await;
        let _ = create_user_like_table(conn).await;
        let _ = create_user_collection_table(conn).await;
        let _ = create_user_status_table(conn).await;
        let _ = create_burrow_table(conn).await;
        let _ = create_user_follow_table(conn).await;
        // match t {
        //     Ok(_) => {}
        //     Err(e) => {
        //         println!("{}", e);
        //     }
        // }
        // let _ = alter_image_table(conn).await;
        Ok(rocket)
    }

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

    async fn create_user_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::user::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::user::Column::Uid)
                    .big_integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(db::user::Column::Username).text().not_null())
            .col(ColumnDef::new(db::user::Column::Password).text().not_null())
            .col(ColumnDef::new(db::user::Column::Email).text().not_null())
            .col(
                ColumnDef::new(db::user::Column::CreateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(db::user::Column::Salt).text().not_null())
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_user_index_username(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = Index::create()
            .name("idx-username")
            .table(db::user::Entity)
            .col(db::user::Column::Username)
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_user_index_email(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = Index::create()
            .name("idx-email")
            .table(db::user::Entity)
            .col(db::user::Column::Email)
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_image_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::image::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::image::Column::Filename)
                    .text()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(db::image::Column::UserId)
                    .big_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(db::image::Column::Size).integer().not_null())
            .col(
                ColumnDef::new(db::image::Column::CreateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::image::Column::LastDownloadTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned();
        // println!("image table: {}", stmt.to_string(PostgresQueryBuilder));
        build_statement(db, &stmt).await
    }

    async fn create_user_status_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::user_status::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::user_status::Column::Uid)
                    .big_integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(db::user_status::Column::UpdateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_status::Column::UserState)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::user_status::Column::ValidBurrow)
                    .text()
                    .not_null()
                    .default("".to_string()),
            )
            .col(
                ColumnDef::new(db::user_status::Column::BannedBurrow)
                    .text()
                    .not_null()
                    .default("".to_string()),
            )
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_content_post_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::content_post::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::content_post::Column::PostId)
                    .extra("bigserial".to_string())
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::Title)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::BurrowId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::CreateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::UpdateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::Section)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::Tag)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_post::Column::PostLen)
                    .integer()
                    .not_null()
                    .default(1),
            )
            .col(
                ColumnDef::new(db::content_post::Column::PostType)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::content_post::Column::PostState)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::content_post::Column::LikeNum)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::content_post::Column::CollectionNum)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .to_owned();
        // println!("user table: {}", stmt.to_string(PostgresQueryBuilder));
        build_statement(db, &stmt).await
    }

    async fn create_content_reply_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::content_reply::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::content_reply::Column::PostId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::ReplyId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::BurrowId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::CreateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::UpdateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::Content)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::content_reply::Column::ReplyState)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .primary_key(
                Index::create()
                    .col(db::content_reply::Column::PostId)
                    .col(db::content_reply::Column::ReplyId),
            )
            .to_owned();
        // println!("user table: {}", stmt.to_string(PostgresQueryBuilder));
        build_statement(db, &stmt).await
    }

    async fn create_user_like_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::user_like::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::user_like::Column::Uid)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_like::Column::PostId)
                    .big_integer()
                    .not_null(),
            )
            .primary_key(
                Index::create()
                    .col(db::user_like::Column::Uid)
                    .col(db::user_like::Column::PostId),
            )
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_user_collection_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::user_collection::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::user_collection::Column::Uid)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_collection::Column::PostId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_collection::Column::IsUpdate)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .primary_key(
                Index::create()
                    .col(db::user_collection::Column::Uid)
                    .col(db::user_collection::Column::PostId),
            )
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_burrow_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::burrow::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::burrow::Column::BurrowId)
                    .extra("bigserial".to_string())
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(db::burrow::Column::Title).text().not_null())
            .col(
                ColumnDef::new(db::burrow::Column::Description)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::burrow::Column::Uid)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::burrow::Column::BurrowState)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::burrow::Column::PostNum)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::burrow::Column::CreateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::burrow::Column::UpdateTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::burrow::Column::Credit)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(db::burrow::Column::Badge)
                    .text()
                    .not_null()
                    .default("".to_string()),
            )
            .col(
                ColumnDef::new(db::burrow::Column::Avatar)
                    .text()
                    .not_null()
                    .default("default.jpg".to_string()),
            )
            .to_owned();
        build_statement(db, &stmt).await
    }

    async fn create_user_follow_table(db: &DbConn) -> Result<ExecResult, DbErr> {
        let stmt = sea_query::Table::create()
            .table(db::user_follow::Entity)
            .if_not_exists()
            .col(
                ColumnDef::new(db::user_follow::Column::Uid)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_follow::Column::BurrowId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(db::user_follow::Column::IsUpdate)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .primary_key(
                Index::create()
                    .col(db::user_follow::Column::Uid)
                    .col(db::user_follow::Column::BurrowId),
            )
            .to_owned();
        build_statement(db, &stmt).await
    }

    // async fn alter_image_table(db: &DbConn) -> Result<ExecResult, DbErr> {
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
}
