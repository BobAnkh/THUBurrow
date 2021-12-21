//! Module for Database Pool
//!
//! `pool` is a collection of `Database Connection Pool`s that can be used to perform
//! operations on multiple databases, e.g. PostgreSQL, Redis, Pulsar, Typesense, Minio, etc.

use deadpool::managed::{self, Manager, Object, PoolConfig, PoolError};
use deadpool::Runtime;
use pulsar::{
    message::proto, producer, Error as PulsarError, MultiTopicProducer, Pulsar, TokioExecutor,
};
use reqwest;
use rocket::State;
use rocket_db_pools::{rocket::figment::Figment, Config, Database, Error, Pool};
use s3::{bucket::Bucket, creds::Credentials, region::Region, BucketConfiguration};
use sea_orm::{DatabaseConnection, DbErr};
use std::time::Duration;

/// Manager used for managing redis connection pool.
pub trait DeadManager: Manager + Sized + Send + Sync + 'static {
    fn new(config: &Config) -> Result<Self, Self::Error>;
}

impl DeadManager for deadpool_redis::Manager {
    fn new(config: &Config) -> Result<Self, Self::Error> {
        Self::new(config.url.as_str())
    }
}

/// Redis Connection Pool
#[derive(Database)]
#[database("redis")]
pub struct RedisDb(RedisPoolWrapper);

impl RedisDb {
    pub async fn get_redis_con(db: &State<RedisDb>) -> Object<deadpool_redis::Manager> {
        let con_wrapper = db.0.get().await.unwrap();
        con_wrapper
    }
}

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

/// PostgreSQL Connection Pool
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

/// Pulsar Connection Pool
#[derive(Database)]
#[database("pulsar-mq")]
pub struct PulsarSearchProducerMq(PulsarProducerPool);

pub struct PulsarProducerPool {
    pub pulsar: Pulsar<TokioExecutor>,
}

#[rocket::async_trait]
impl Pool for PulsarProducerPool {
    type Connection = MultiTopicProducer<TokioExecutor>;
    type Error = PulsarError;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config: Config = figment.extract().unwrap();
        let pulsar = Pulsar::builder(&config.url, TokioExecutor).build().await?;
        Ok(PulsarProducerPool { pulsar })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self
            .pulsar
            .producer()
            .with_options(producer::ProducerOptions {
                schema: Some(proto::Schema {
                    r#type: proto::schema::Type::String as i32,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .build_multi_topic())
    }
}

// #[rocket::async_trait]
// pub trait RocketPulsarProducer {
//     async fn get_producer(
//         &self,
//         topic: &str,
//     ) -> Result<producer::Producer<TokioExecutor>, PulsarError>;
// }

// #[rocket::async_trait]
// impl RocketPulsarProducer for Pulsar<TokioExecutor> {
//     async fn get_producer(
//         &self,
//         topic: &str,
//     ) -> Result<producer::Producer<TokioExecutor>, PulsarError> {
//         self.producer()
//             .with_topic(topic)
//             .with_options(producer::ProducerOptions {
//                 schema: Some(proto::Schema {
//                     r#type: proto::schema::Type::String as i32,
//                     ..Default::default()
//                 }),
//                 ..Default::default()
//             })
//             .build()
//             .await
//     }
// }

/// Minio Connection Pool
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
        let info: Vec<&str> = config.url.split("://").collect();
        let info = info[1];
        let info: Vec<&str> = info.split('@').collect();
        let mut user: Option<String> = None;
        let mut password: Option<String> = None;
        let host: String;
        if info.len() == 1 {
            host = "http://".to_string() + info[0];
        } else if info.len() == 2 {
            let user_info: Vec<&str> = info[0].split(':').collect();
            user = Some(user_info[0].to_string());
            password = Some(user_info[1].to_string());
            host = "http://".to_string() + info[1];
        } else {
            panic!("Invalid minio url.");
        }
        let bucket_name = "thuburrow-image";
        let bucket_region = Region::Custom {
            region: "".to_string(),
            endpoint: host,
        };
        let bucket_credentials = Credentials {
            access_key: user,
            secret_key: password,
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

/// Typesense Connection Pool
#[derive(Database)]
#[database("search")]
pub struct TypesenseSearch(TypesenseSearchPool);

pub struct TypesenseSearchPool {
    pub search_client: SearchClient,
}

#[derive(Clone)]
pub struct SearchClient {
    pub client: reqwest::Client,
    pub typesense_addr: String,
    pub typesense_api_key: String,
}

#[rocket::async_trait]
impl Pool for TypesenseSearchPool {
    type Connection = SearchClient;
    type Error = std::convert::Infallible;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config: Config = figment.extract().unwrap();
        let info: Vec<&str> = config.url.split('@').collect();
        let addr: String;
        let api_key: String;
        if info.len() == 1 {
            addr = info[0].to_owned();
            api_key = "8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_owned();
        } else if info.len() == 2 {
            addr = info[0].to_owned();
            api_key = info[1].to_owned();
        } else {
            panic!("Invalid typesense url.");
        }
        Ok(TypesenseSearchPool {
            search_client: SearchClient {
                client: reqwest::Client::new(),
                typesense_addr: addr,
                typesense_api_key: api_key,
            },
        })
    }

    async fn get(&self) -> Result<Self::Connection, Self::Error> {
        Ok(self.search_client.clone())
    }
}

pub trait Search {
    fn build_post(&self, uri: &str) -> reqwest::RequestBuilder;
    fn build_get(&self, uri: &str) -> reqwest::RequestBuilder;
}

impl Search for SearchClient {
    fn build_post(&self, uri: &str) -> reqwest::RequestBuilder {
        self.client
            .post(self.typesense_addr.to_owned() + uri)
            .header("Content-Type", "application/json")
            .header("X-TYPESENSE-API-KEY", &self.typesense_api_key)
    }
    fn build_get(&self, uri: &str) -> reqwest::RequestBuilder {
        self.client
            .get(self.typesense_addr.to_owned() + uri)
            .header("X-TYPESENSE-API-KEY", &self.typesense_api_key)
    }
}
