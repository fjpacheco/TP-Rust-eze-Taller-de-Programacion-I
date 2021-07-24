use crate::{
    commands::{check_error_cases_without_elements, Runnable},
    messages::redis_messages,
    native_types::ErrorStruct,
    native_types::{RSimpleString, RedisType},
    Database,
};
pub struct FlushDb;
use crate::native_types::error_severity::ErrorSeverity;
use std::sync::{Arc, Mutex};
impl Runnable<Arc<Mutex<Database>>> for FlushDb {
    /// Delete all the keys of the currently selected DB. This command never fails.
    ///
    /// # Return value
    /// [String] _encoded_ in [RSimpleString]: OK if SET was executed correctly.
    ///
    /// # Error
    /// Return an [ErrorStruct] if:
    ///
    /// * Buffer [Vec]<[String]> is received empty, or not received with only one element.
    /// * [Database] received in <[Arc]<[Mutex]>> is poisoned.       
    fn run(
        &self,
        buffer: Vec<String>,
        database: &mut Arc<Mutex<Database>>,
    ) -> Result<String, ErrorStruct> {
        let mut database = database.lock().map_err(|_| {
            ErrorStruct::from(redis_messages::poisoned_lock(
                "database",
                ErrorSeverity::ShutdownServer,
            ))
        })?;
        check_error_cases_without_elements(&buffer, "flushdb", 1)?;

        database.clear();
        Ok(RSimpleString::encode("OK".to_string()))
    }
}
