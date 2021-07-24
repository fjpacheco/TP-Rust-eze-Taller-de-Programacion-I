use crate::tcp_protocol::server_redis_attributes::ServerRedisAttributes;
use crate::{
    commands::{check_empty, Runnable},
    messages::redis_messages,
    native_types::{ErrorStruct, RSimpleString, RedisType},
};

pub struct ConfigSetLogFile;

impl Runnable<ServerRedisAttributes> for ConfigSetLogFile {
    fn run(
        &self,
        buffer: Vec<String>,
        server: &mut ServerRedisAttributes,
    ) -> Result<String, ErrorStruct> {
        check_empty(&buffer, "config set logfile")?;

        let new_file_name = buffer.get(0).unwrap().to_string(); // no empty!
        server.change_logfilename(new_file_name)?;
        Ok(RSimpleString::encode(redis_messages::ok()))
    }
}
