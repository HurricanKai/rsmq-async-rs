use crate::functions::RsmqFunctions;
use crate::r#trait::RsmqConnection;
use crate::types::{RedisBytes, RsmqMessage, RsmqOptions, RsmqQueueAttributes};
use crate::RsmqResult;
use core::convert::TryFrom;
use core::marker::PhantomData;

#[derive(Clone)]
struct RedisConnection(redis::aio::MultiplexedConnection);

impl std::fmt::Debug for RedisConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RedisAsyncConnnection")
    }
}

#[derive(Debug, Clone)]
pub struct MultiplexedRsmq {
    connection: RedisConnection,
    functions: RsmqFunctions<redis::aio::MultiplexedConnection>,
}

impl MultiplexedRsmq {
    /// Creates a new RSMQ instance, including its connection
    pub async fn new(options: RsmqOptions) -> RsmqResult<MultiplexedRsmq> {
        let password = if let Some(password) = options.password.clone() {
            format!("redis:{}@", password)
        } else {
            "".to_string()
        };

        let url = format!(
            "redis://{}{}:{}/{}",
            password, options.host, options.port, options.db
        );

        let client = redis::Client::open(url)?;

        let connection = client.get_multiplexed_async_connection().await?;

        Ok(MultiplexedRsmq::new_with_connection(options, connection))
    }

    /// Special method for when you already have a redis-rs connection and you don't want redis_async to create a new one.
    pub fn new_with_connection(
        options: RsmqOptions,
        connection: redis::aio::MultiplexedConnection,
    ) -> MultiplexedRsmq {
        MultiplexedRsmq {
            connection: RedisConnection(connection),
            functions: RsmqFunctions {
                ns: options.ns.clone(),
                realtime: options.realtime,
                conn: PhantomData,
            },
        }
    }
}

#[async_trait::async_trait]
impl RsmqConnection for MultiplexedRsmq {
    async fn change_message_visibility(
        &mut self,
        qname: &str,
        message_id: &str,
        seconds_hidden: u64,
    ) -> RsmqResult<()> {
        self.functions
            .change_message_visibility(&mut self.connection.0, qname, message_id, seconds_hidden)
            .await
    }

    async fn create_queue(
        &mut self,
        qname: &str,
        seconds_hidden: Option<u32>,
        delay: Option<u32>,
        maxsize: Option<i32>,
    ) -> RsmqResult<()> {
        self.functions
            .create_queue(
                &mut self.connection.0,
                qname,
                seconds_hidden,
                delay,
                maxsize,
            )
            .await
    }

    async fn delete_message(&mut self, qname: &str, id: &str) -> RsmqResult<bool> {
        self.functions
            .delete_message(&mut self.connection.0, qname, id)
            .await
    }
    async fn delete_queue(&mut self, qname: &str) -> RsmqResult<()> {
        self.functions
            .delete_queue(&mut self.connection.0, qname)
            .await
    }
    async fn get_queue_attributes(&mut self, qname: &str) -> RsmqResult<RsmqQueueAttributes> {
        self.functions
            .get_queue_attributes(&mut self.connection.0, qname)
            .await
    }

    async fn list_queues(&mut self) -> RsmqResult<Vec<String>> {
        self.functions.list_queues(&mut self.connection.0).await
    }

    async fn pop_message<E: TryFrom<RedisBytes, Error = Vec<u8>>>(
        &mut self,
        qname: &str,
    ) -> RsmqResult<Option<RsmqMessage<E>>> {
        self.functions
            .pop_message::<E>(&mut self.connection.0, qname)
            .await
    }

    async fn receive_message<E: TryFrom<RedisBytes, Error = Vec<u8>>>(
        &mut self,
        qname: &str,
        seconds_hidden: Option<u64>,
    ) -> RsmqResult<Option<RsmqMessage<E>>> {
        self.functions
            .receive_message::<E>(&mut self.connection.0, qname, seconds_hidden)
            .await
    }

    async fn send_message<E: Into<RedisBytes> + Send>(
        &mut self,
        qname: &str,
        message: E,
        delay: Option<u64>,
    ) -> RsmqResult<String> {
        self.functions
            .send_message(&mut self.connection.0, qname, message, delay)
            .await
    }

    async fn set_queue_attributes(
        &mut self,
        qname: &str,
        seconds_hidden: Option<u64>,
        delay: Option<u64>,
        maxsize: Option<i64>,
    ) -> RsmqResult<RsmqQueueAttributes> {
        self.functions
            .set_queue_attributes(
                &mut self.connection.0,
                qname,
                seconds_hidden,
                delay,
                maxsize,
            )
            .await
    }
}
