use redis::RedisResult;

pub(crate) fn get_connection() -> Result<redis::Connection, redis::RedisError>{
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_connection()
}

pub(crate) fn default_save<'a>(con: &mut redis::Connection, key_prefix: &str,key_id: &str, val: impl serde::Serialize) ->  redis::RedisResult<()> {
    let stringified = serde_json::to_string(&val).unwrap();
    let key = format!(r"{}:{}", key_prefix, key_id);
    //Add to a user set here
    //let result = redis::cmd("SADD").arg(key).arg(stringified).query(con);
    let result = redis::cmd("SET").arg(key).arg(stringified).query(con);
    result
}

pub(crate) fn default_fetch< T: for<'a> serde::Deserialize<'a>>( con: & mut redis::Connection,key: impl Into<String> + redis::ToRedisArgs ,) -> Option<T>{

    let item_str: RedisResult<String> = redis::cmd("GET").arg(key).query(con);
    if item_str.is_err() {
        return None
    }
    let res = item_str.unwrap();
    let item_json = serde_json::from_str(&res);
    if item_json.is_err() {
        return None
    }
    Some(item_json.unwrap())
}
pub(crate) fn default_fetch_from_uuid< T: for<'a> serde::Deserialize<'a>>( con: & mut redis::Connection,key_prefix: &str, id: impl Into<String> ,) -> Option<T>{
    let key = format!(r"{}:{}", key_prefix,&id.into(), );
    default_fetch(con, key)
}

pub(crate) fn default_fetch_all<T: for<'a> serde::Deserialize<'a>>(con: &mut redis::Connection, key_prefix: &str) -> Vec<T> {
    let key = format!(r"{}:*", key_prefix);
    // Hier wird noch der default_fetch_all verwendet, der nur die keys holt
    // sollte mal gegen den aktuellen user getauschtwerden, der dann die keys mit SMEMBERS holt
    let keys: Vec<String> = redis::Cmd::keys(key).query(con).unwrap();
    let mut items: Vec<T> = vec![];
    for key in keys {
        let item = default_fetch(con, key);
        if item.is_some() {
            items.push(item.unwrap());
        }
    }
    items
}

