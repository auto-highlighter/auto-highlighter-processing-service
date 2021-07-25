
use deadpool_lapin::{Manager, PoolError};
use dotenv;
use envy;
use futures::{join, StreamExt};
use lapin::{options::*, types::FieldTable, BasicProperties, ConnectionProperties};
use serde::{Deserialize};
use serde_json::from_slice;
use std::result::Result as StdResult;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio_amqp::*;
use super::gst_highlighter::{HighlightConfig, make_highlights};
use std::thread::spawn;

type RMQResult<T> = StdResult<T, PoolError>;
type Result<T> = StdResult<T, Error>;

type Pool = deadpool::managed::Pool<lapin::Connection, lapin::Error>;
type Connection = deadpool::managed::Object<lapin::Connection, lapin::Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug)]
struct RMQConfig {
    amqp_url: String,
    pool_size: usize,
}

pub async fn rmq_runner() {
    dotenv::dotenv().expect("Failed to read .env file");
    let config = match envy::from_env::<RMQConfig>() {
        Ok(config) => config,
        Err(e) => panic!("Couldn't read RMQConfig ({})", e),
    };

    let manager = Manager::new(
        config.amqp_url,
        ConnectionProperties::default().with_tokio(),
    );
    let pool = deadpool::managed::Pool::new(manager, config.pool_size);

    let _ = join!(rmq_listen(pool.clone()));
}

async fn publish(pool: Pool) -> Result<()> {
    let payload = b"Hello world!";

    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("can't connect to rmq, {}", e);
        Error::RMQPoolError(e)
    })?;

    let channel = rmq_con.create_channel().await.map_err(|e| {
        eprintln!("can't create channel, {}", e);
        Error::RMQError(e)
    })?;

    channel
        .basic_publish(
            "",
            "highlighting",
            BasicPublishOptions::default(),
            payload.to_vec(),
            BasicProperties::default(),
        )
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            Error::RMQError(e)
        })?
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            Error::RMQError(e)
        })?;
    Ok(())
}

async fn get_rmq_con(pool: Pool) -> RMQResult<Connection> {
    let connection = pool.get().await?;
    Ok(connection)
}

async fn rmq_listen(pool: Pool) -> Result<()> {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("connecting rmq consumer...");
        match init_rmq_listen(pool.clone()).await {
            Ok(_) => println!("rmq listen returned"),
            Err(e) => eprintln!("rmq listen had an error: {}", e),
        };
    }
}

async fn init_rmq_listen(pool: Pool) -> Result<()> {
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("could not get rmq con: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    let queue = channel
        .queue_declare(
            "highlighting",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue);

    let mut consumer = channel
        .basic_consume(
            "highlighting",
            "gstcomsumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("rmq consumer connected, waiting for messages");
    while let Some(delivery) = consumer.next().await {
        if let Ok((channel, delivery)) = delivery {
            let payload = from_slice::<HighlightConfig>(&delivery.data)?;
            println!("received msg: {:?}", payload);
            spawn(|| {
                make_highlights(payload);
            });
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?
        }
    }
    Ok(())
}
