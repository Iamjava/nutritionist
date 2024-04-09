pub(crate) mod connector;

#[cfg(test)]
mod test {
    use redis::Commands;
    use crate::db::connector;
    #[tokio::test]
    async fn test_db() {
        let mut con = connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        //let _: () = con.set("my_key", 43).expect("Could not set key,maybe redis is not running");
        let key: i32 = con.get("my_key").expect("Could not get key,maybe redis is not running");
        assert_eq!(key, 43)
    }

}