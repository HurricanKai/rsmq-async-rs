mod support;

use rsmq_async::{RedisBytes, Rsmq, RsmqConnection, RsmqError};
use std::convert::TryFrom;
use support::*;

#[test]
fn send_receiving_deleting_message() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue1", None, None, None).await.unwrap();

        rsmq.send_message("queue1", "testmessage", None)
            .await
            .unwrap();

        let message = rsmq
            .receive_message::<String>("queue1", None)
            .await
            .unwrap();
        assert!(message.is_some());

        let message = message.unwrap();

        rsmq.delete_message("queue1", &message.id).await.unwrap();

        assert_eq!(message.message, "testmessage".to_string());

        let message = rsmq
            .receive_message::<String>("queue1", None)
            .await
            .unwrap();

        assert!(message.is_none());
        rsmq.delete_queue("queue1").await.unwrap();
    })
}

#[test]
fn send_receiving_deleting_message_vec_u8() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue1", None, None, None).await.unwrap();

        rsmq.send_message("queue1", "testmessage", None)
            .await
            .unwrap();

        let message = rsmq
            .receive_message::<Vec<u8>>("queue1", None)
            .await
            .unwrap();
        assert!(message.is_some());

        let message = message.unwrap();

        rsmq.delete_message("queue1", &message.id).await.unwrap();

        assert_eq!(message.message, b"testmessage");

        let message = rsmq
            .receive_message::<Vec<u8>>("queue1", None)
            .await
            .unwrap();

        assert!(message.is_none());
        rsmq.delete_queue("queue1").await.unwrap();
    })
}

#[test]
fn send_receiving_deleting_message_custom_type() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    #[derive(Debug, PartialEq)]
    struct MyValue(Vec<u8>);

    impl TryFrom<RedisBytes> for MyValue {
        type Error = Vec<u8>;

        fn try_from(t: RedisBytes) -> Result<Self, Self::Error> {
            Ok(MyValue(t.into_bytes()))
        }
    }

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue1", None, None, None).await.unwrap();

        rsmq.send_message("queue1", b"testmessage".to_owned().to_vec(), None)
            .await
            .unwrap();

        let message = rsmq
            .receive_message::<MyValue>("queue1", None)
            .await
            .unwrap();
        assert!(message.is_some());

        let message = message.unwrap();

        rsmq.delete_message("queue1", &message.id).await.unwrap();

        assert_eq!(message.message, MyValue(b"testmessage".to_owned().to_vec()));

        let message = rsmq
            .receive_message::<MyValue>("queue1", None)
            .await
            .unwrap();

        assert!(message.is_none());
        rsmq.delete_queue("queue1").await.unwrap();
    })
}

#[test]
fn pop_message() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue2", None, None, None).await.unwrap();

        rsmq.send_message("queue2", "testmessage", None)
            .await
            .unwrap();

        let message = rsmq.pop_message::<String>("queue2").await.unwrap();

        assert!(message.is_some());

        let message = message.unwrap();

        assert_eq!(message.message, "testmessage");

        let message = rsmq.pop_message::<String>("queue2").await.unwrap();

        assert!(message.is_none());

        rsmq.delete_queue("queue2").await.unwrap();
    })
}

#[test]
fn pop_message_vec_u8() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue2", None, None, None).await.unwrap();

        rsmq.send_message("queue2", "testmessage", None)
            .await
            .unwrap();

        let message = rsmq.pop_message::<Vec<u8>>("queue2").await.unwrap();

        assert!(message.is_some());

        let message = message.unwrap();

        assert_eq!(message.message, "testmessage".as_bytes());

        let message = rsmq.pop_message::<String>("queue2").await.unwrap();

        assert!(message.is_none());

        rsmq.delete_queue("queue2").await.unwrap();
    })
}

#[test]
fn creating_queue() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue3", None, None, None).await.unwrap();

        let queues = rsmq.list_queues().await.unwrap();

        assert_eq!(queues, vec!("queue3"));

        let result = rsmq.create_queue("queue3", None, None, None).await;

        assert!(result.is_err());

        if let Err(RsmqError::QueueExists) = result {
            rsmq.delete_queue("queue3").await.unwrap();
        } else {
            panic!()
        }
    })
}

#[test]
fn updating_queue() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue4", None, None, None).await.unwrap();

        let attributes = rsmq.get_queue_attributes("queue4").await.unwrap();

        assert_eq!(attributes.vt, 30);
        assert_eq!(attributes.delay, 0);
        assert_eq!(attributes.maxsize, 65536);
        assert_eq!(attributes.totalrecv, 0);
        assert_eq!(attributes.totalsent, 0);
        assert_eq!(attributes.msgs, 0);
        assert_eq!(attributes.hiddenmsgs, 0);
        assert!(attributes.created > 0);
        assert!(attributes.modified > 0);

        rsmq.set_queue_attributes("queue4", Some(45), Some(5), Some(2048))
            .await
            .unwrap();

        let attributes = rsmq.get_queue_attributes("queue4").await.unwrap();

        assert_eq!(attributes.vt, 45);
        assert_eq!(attributes.delay, 5);
        assert_eq!(attributes.maxsize, 2048);
        assert_eq!(attributes.totalrecv, 0);
        assert_eq!(attributes.totalsent, 0);
        assert_eq!(attributes.msgs, 0);
        assert_eq!(attributes.hiddenmsgs, 0);
        assert!(attributes.created > 0);
        assert!(attributes.modified > 0);

        rsmq.delete_queue("queue4").await.unwrap();
    })
}

#[test]
fn deleting_queue() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue5", None, None, None).await.unwrap();

        let queues = rsmq.list_queues().await.unwrap();

        assert_eq!(queues, vec!("queue5"));

        rsmq.delete_queue("queue5").await.unwrap();

        let queues = rsmq.list_queues().await.unwrap();

        assert_eq!(queues, Vec::<String>::new());

        let result = rsmq.delete_queue("queue5").await;

        assert!(result.is_err());

        if let Err(RsmqError::QueueNotFound) = result {
        } else {
            panic!("{:?}", result)
        }

        let result = rsmq.get_queue_attributes("queue5").await;

        assert!(result.is_err());

        if let Err(RsmqError::QueueNotFound) = result {
        } else {
            panic!("{:?}", result)
        }

        let result = rsmq
            .set_queue_attributes("queue5", Some(45), Some(5), Some(2048))
            .await;

        assert!(result.is_err());

        if let Err(RsmqError::QueueNotFound) = result {
        } else {
            panic!("{:?}", result)
        }
    })
}

#[test]
fn change_message_visibility() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        let ctx = TestContext::new();
        let connection = ctx.async_connection().await.unwrap();
        let mut rsmq = Rsmq::new_with_connection(Default::default(), connection);

        rsmq.create_queue("queue6", None, None, None).await.unwrap();

        rsmq.send_message("queue6", "testmessage", None)
            .await
            .unwrap();

        let message = rsmq
            .receive_message::<String>("queue6", None)
            .await
            .unwrap();
        assert!(message.is_some());

        let message_id = message.unwrap().id;

        let message = rsmq
            .receive_message::<String>("queue6", None)
            .await
            .unwrap();
        assert!(message.is_none());

        rsmq.change_message_visibility("queue6", &message_id, 0)
            .await
            .unwrap();

        let ten_millis = std::time::Duration::from_millis(10);
        std::thread::sleep(ten_millis);

        let message = rsmq
            .receive_message::<String>("queue6", None)
            .await
            .unwrap();
        assert!(message.is_some());

        assert_eq!(message_id, message.unwrap().id);

        rsmq.delete_queue("queue6").await.unwrap();
    })
}
