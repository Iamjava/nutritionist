use redis::{Connection, RedisResult};
use std::fmt::format;

pub(crate) fn get_connection() -> Result<redis::Connection, redis::RedisError> {
    let redis_url = std::env::var("REDIS_HOST").unwrap_or("127.0.0.1/".to_string());
    let client = redis::Client::open("redis://".to_owned() + &*redis_url)?;
    client.get_connection()
}

pub(crate) fn default_save_expire<'a>(
    con: &mut redis::Connection,
    key_prefix: &str,
    key_id: &str,
    val: impl serde::Serialize,
    expiry: i32,
) -> redis::RedisResult<()> {
    let stringified = serde_json::to_string(&val).unwrap();
    let key = format!(r"{}:{}", key_prefix, key_id);
    //Add to a user set here
    //let result = redis::cmd("SADD").arg(key).arg(stringified).query(con);
    let result = redis::cmd("SET")
        .arg(key.clone())
        .arg(stringified)
        .query(con);
    let result_ex: RedisResult<()> = redis::cmd("EXPIRE").arg(key.clone()).arg(expiry).query(con);
    result
}
pub(crate) fn default_save<'a>(
    con: &mut redis::Connection,
    key_prefix: &str,
    key_id: &str,
    val: impl serde::Serialize,
) -> redis::RedisResult<()> {
    let stringified = serde_json::to_string(&val).unwrap();
    let key = format!(r"{}:{}", key_prefix, key_id);
    //Add to a user set here
    //let result = redis::cmd("SADD").arg(key).arg(stringified).query(con);
    let result = redis::cmd("SET").arg(key).arg(stringified).query(con);
    result
}

pub(crate) fn default_fetch(
    con: &mut redis::Connection,
    key: impl Into<String> + redis::ToRedisArgs,
) -> Result<Option<String>, redis::RedisError> {
    let item_str: RedisResult<String> = redis::cmd("GET").arg(key).query(con);
    if item_str.is_err() {
        return Ok(None);
    }
    let res = item_str.unwrap();
    Ok(Some(res))
}
pub(crate) fn default_fetch_and_parse<T: for<'a> serde::Deserialize<'a> + std::fmt::Debug>(
    con: &mut redis::Connection,
    key: impl Into<String> + redis::ToRedisArgs,
) -> Result<Option<T>, redis::RedisError> {
    let item_str = default_fetch(con, key);
    let res = item_str?.ok_or(redis::RedisError::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not fetch from redis",
    )))?;
    let item_json = serde_json::from_str(&res);
    Ok(Some(item_json.unwrap()))
}
pub(crate) fn default_fetch_from_uuid<T: for<'a> serde::Deserialize<'a> + std::fmt::Debug>(
    con: &mut redis::Connection,
    key_prefix: &str,
    id: impl Into<String>,
) -> Option<T> {
    let key = format!(r"{}:{}", key_prefix, &id.into(),);
    default_fetch_and_parse(con, key).unwrap_or(None)
}

pub(crate) fn default_fetch_all<T: for<'a> serde::Deserialize<'a> + std::fmt::Debug>(
    con: &mut redis::Connection,
    key_prefix: &str,
) -> Vec<T> {
    let key = format!(r"{}:*", key_prefix);
    // Hier wird noch der default_fetch_all verwendet, der nur die keys holt
    // sollte mal gegen den aktuellen user getauschtwerden, der dann die keys mit SMEMBERS holt
    let keys: Vec<String> = redis::Cmd::keys(key).query(con).unwrap();
    let mut items: Vec<T> = vec![];
    for key in keys {
        let item: Option<T> =
            default_fetch_and_parse(con, key).expect("Could not fetch from redis");
        if item.is_some() {
            items.push(item.unwrap());
        }
    }
    items
}

pub(crate) fn default_fetch_all_keys(con: &mut redis::Connection, key_prefix: &str) -> Vec<String> {
    let key = format!(r"{}:*", key_prefix);
    let keys: Vec<String> = redis::Cmd::keys(key).query(con).unwrap();
    keys
}

pub(crate) fn add_to_set(
    con: &mut redis::Connection,
    key: String,
    val: String,
) -> redis::RedisResult<()> {
    redis::cmd("SADD").arg(key).arg(val).query(con)
}

pub(crate) fn get_set(con: &mut redis::Connection, key: String) -> Vec<String> {
    let keys: Vec<String> = redis::Cmd::smembers(key).query(con).unwrap();
    keys
}

pub(crate) fn remove_from_set(p0: &mut Connection, p1: String, p2: String) -> RedisResult<()> {
    redis::cmd("SREM").arg(p1).arg(p2).query(p0)
}
