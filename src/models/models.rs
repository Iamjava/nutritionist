use redis::{ RedisResult};
use crate::open_food_facts;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait RedisORM {
    fn save(&self,con:&mut redis::Connection) -> RedisResult<()>;
    fn fetch_from_uuid( con: & mut redis::Connection, id: &str,) -> Option<Self> where Self: Sized;
    fn default() -> Self where Self: Sized;
    fn all(con: &mut redis::Connection) -> Vec<Self> where Self: Sized;
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub(crate) struct Meal {
    pub(crate) contents: Vec<open_food_facts::models::Product>,
    pub(crate) id: Uuid,
}


#[derive(Debug, Serialize, Deserialize,Clone)]
pub(crate) struct NutritionistSearchQuery {
    pub(crate) query: String,
}
