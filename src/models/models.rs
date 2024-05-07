use crate::db::connector::{default_fetch_all, default_fetch_from_uuid, default_save};
use redis::RedisResult;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait RedisORM {
    fn save(&self, con: &mut redis::Connection) -> RedisResult<()>
    where
        Self: Serialize,
    {
        default_save(con, &Self::redis_type_name(), &self.redis_id(), self)
    }
    fn fetch_from_uuid(con: &mut redis::Connection, id: &str) -> Option<Self>
    where
        Self: Sized,
        for<'a> Self: Deserialize<'a>,
        Self: Debug,
    {
        default_fetch_from_uuid(con, Self::redis_type_name().as_str(), id)
    }
    fn example() -> Self
    where
        Self: Sized;
    fn all(con: &mut redis::Connection) -> Vec<Self>
    where
        Self: Sized,
        for<'a> Self: Deserialize<'a>,
        Self: Debug,
    {
        default_fetch_all(con, Self::redis_type_name().as_str())
    }
    fn redis_type_name() -> String;

    fn redis_id(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct NutritionistSearchQuery {
    pub(crate) query: String,
}
