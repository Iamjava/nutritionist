use redis::{Connection, RedisResult};
use crate::open_food_facts;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::connector::{default_fetch_all, default_fetch_from_uuid, default_save};

pub trait RedisInterface {
    fn save(&self,con:&mut redis::Connection) -> RedisResult<()>;
    fn fetch_from_uuid( con: & mut redis::Connection, id: &str,) -> Option<Self> where Self: Sized;
    fn new() -> Self where Self: Sized;
    fn all(con: &mut redis::Connection) -> Vec<Self> where Self: Sized;
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub(crate) struct Meal {
    pub(crate) contents: Vec<open_food_facts::models::Product>,
    pub(crate) id: Uuid,
}
impl RedisInterface for Meal {
    fn save(&self,con: & mut redis::Connection) -> redis::RedisResult<()> {
        default_save(con, "meal", &self.id.to_string(), self)
    }

    fn fetch_from_uuid(con: & mut redis::Connection, id: &str,) -> Option<Meal>{
        default_fetch_from_uuid(con, "meal", id)
    }

    fn new() -> Meal {
        Meal {
            contents: vec![],
            id: Uuid::new_v4(),
        }
    }

    fn all(con: &mut Connection) -> Vec<Self> where Self: Sized {
        default_fetch_all(con, "meal")
    }
}

