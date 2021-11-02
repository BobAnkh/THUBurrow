use deadpool::managed::{self, Manager, Object, PoolConfig, PoolError};
use deadpool::Runtime;
use rocket_db_pools::{rocket::figment::Figment, Config, Database, Error, Pool};
use s3::BucketConfiguration;
use sea_orm::{DatabaseConnection, DbErr};
use std::time::Duration;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

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

#[derive(Database)]
#[database("minio")]
pub struct MinioImageStorage(MinioImagePool);

pub struct MinioImagePool {
    pub connection: Bucket,
}

#[rocket::async_trait]
impl Pool for MinioImagePool {
    type Connection = Bucket;
    type Error = std::convert::Infallible;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config: Config = figment.extract().unwrap();
        let bucket_name = "thuburrow-image";
        let bucket_region = Region::Custom {
            region: "".to_string(),
            endpoint: config.url.to_string(),
        };
        // TODO(config): should pass config from outside
        let bucket_credentials = Credentials {
            access_key: Some("minio".to_owned()),
            secret_key: Some("miniopassword".to_owned()),
            security_token: None,
            session_token: None,
        };
        // instantiate the bucket
        let bucket = Bucket::new_with_path_style(bucket_name, bucket_region, bucket_credentials)
            .expect("Can not instantiate bucket");
        // create a new bucket if not already exists
        let (_, code) = bucket.head_object("/").await.unwrap();
        if code == 404 {
            match Bucket::create_with_path_style(
                bucket.name.as_str(),
                bucket.region.clone(),
                bucket.credentials.clone(),
                BucketConfiguration::default(),
            )
            .await
            {
                Ok(create_result) => println!(
                    "Bucket {} created! {} - {}",
                    bucket.name, create_result.response_code, create_result.response_text
                ),
                Err(e) => panic!("Can not create bucket: {}", e),
            }
        }
        Ok(MinioImagePool { connection: bucket })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.connection.clone())
    }
}
