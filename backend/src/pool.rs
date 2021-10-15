use rocket_db_pools::{rocket::figment::Figment, Config, Database, Pool};
use sea_orm::{DatabaseConnection, DbErr};

#[derive(Database)]
#[database("backend")]
pub struct Db(SeaOrmPool);

#[derive(Debug)]
pub struct SeaOrmPool {
    pub connection: DatabaseConnection,
}

#[rocket::async_trait]
impl Pool for SeaOrmPool {
    type Connection = DatabaseConnection;
    type Error = DbErr;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config: Config = figment.extract().unwrap();
        let connection = sea_orm::Database::connect(&config.url).await?;

        Ok(SeaOrmPool { connection })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.connection.clone())
    }

    // async fn borrow(&self) -> &Self::Connection {
    //     &self.connection
    // }
}
