#[cfg(test)]
mod testings_redis {
    extern crate redis;
    use std::thread::{self, JoinHandle};

    use redis::RedisError;
    use redis_rust::{
        native_types::ErrorStruct, redis_config::RedisConfig, tcp_protocol::server::ServerRedis,
    };

    #[test]
    #[ignore] // TODO: Recordar hablar con Pablo al respecto de los Sleeps en Tests.
    fn test01_set_and_get() -> Result<(), ErrorStruct> {
        let _server_thread: JoinHandle<Result<(), ErrorStruct>> = thread::spawn(|| {
            ServerRedis::start(vec![])?;
            Ok(())
        });

        let client = redis::Client::open(
            "redis://".to_owned()
                + &RedisConfig::default().ip()
                + ":"
                + &RedisConfig::default().port()
                + "/",
        )
        .unwrap();

        let mut conection_client = client.get_connection().unwrap();

        let received: Result<String, RedisError> = redis::cmd("set")
            .arg("key")
            .arg("value")
            .query(&mut conection_client);

        assert_eq!(received.unwrap(), "OK");

        let received: Result<String, RedisError> =
            redis::cmd("shutdown").query(&mut conection_client);
        assert_eq!(received.unwrap(), "SERVER OFF");

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_random() {
        let _server_thread: JoinHandle<Result<(), ErrorStruct>> = thread::spawn(|| {
            ServerRedis::start(vec![])?;
            Ok(())
        });

        let client = redis::Client::open(
            "redis://".to_owned()
                + &RedisConfig::default().ip()
                + ":"
                + &RedisConfig::default().port()
                + "/",
        )
        .unwrap();
        let mut conection_client = client.get_connection().unwrap();
        println!("Conectado!");

        let received_3: Result<String, RedisError> = redis::cmd("set")
            .arg("Agustín")
            .arg("Firmapaz")
            .query(&mut conection_client);
        println!("set Agustín Firmapaz => {:?}", received_3);
        let received_3: Result<String, RedisError> = redis::cmd("set")
            .arg("Martina")
            .arg("Panetta")
            .query(&mut conection_client);
        println!("set Martina Panetta => {:?}", received_3);

        let received_4: Result<String, RedisError> = redis::cmd("get")
            .arg("Martina")
            .query(&mut conection_client);
        println!("get Martina => {:?}", received_4);
        let received_4: Result<String, RedisError> = redis::cmd("get")
            .arg("Agustín")
            .query(&mut conection_client);
        println!("get Agustín => {:?}", received_4);
    }
}