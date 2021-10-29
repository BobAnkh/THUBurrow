use deadpool::managed::{self, Manager, Object, PoolConfig, PoolError};
use deadpool::Runtime;
use rocket_db_pools::{rocket::figment::Figment, Config, Database, Error, Pool};
use sea_orm::{DatabaseConnection, DbErr};
use std::time::Duration;

// redis for keydb
pub trait DeadManager: Manager + Sized + Send + Sync + 'static {
    fn new(config: &Config) -> Result<Self, Self::Error>;
}

impl DeadManager for deadpool_redis::Manager {
    fn new(config: &Config) -> Result<Self, Self::Error> {
        Self::new(config.url.as_str())
    }
}

#[derive(Database)]
#[database("keydb")]
pub struct RedisDb(RedisPoolWrapper);

pub struct RedisPoolWrapper<M: Manager = deadpool_redis::Manager, C: From<Object<M>> = Object<M>> {
    pool: managed::Pool<M, C>,
}

#[rocket::async_trait]
impl<M: DeadManager, C: From<Object<M>>> Pool for RedisPoolWrapper<M, C>
where
    M::Type: Send,
    C: Send + Sync + 'static,
    M::Error: std::error::Error,
{
    type Error = Error<M::Error, PoolError<M::Error>>;

    type Connection = C;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config: Config = figment.extract()?;
        let manager = M::new(&config).map_err(Error::Init)?;

        let mut pool_config = PoolConfig::new(config.max_connections);
        pool_config.timeouts.create = Some(Duration::from_secs(config.connect_timeout));
        pool_config.timeouts.wait = Some(Duration::from_secs(config.connect_timeout));
        pool_config.timeouts.recycle = config.idle_timeout.map(Duration::from_secs);
        let pool = managed::Pool::builder(manager)
            .config(pool_config)
            .runtime(Runtime::Tokio1)
            .build()
            .unwrap();
        Ok(RedisPoolWrapper { pool })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        self.pool.get().await.map_err(Error::Get)
    }
}

// sql for postgres
#[derive(Database)]
#[database("pgdb")]
pub struct PgDb(SeaOrmPool);

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
}
