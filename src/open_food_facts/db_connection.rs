use crate::models::models::RedisORM;

#[cfg(test)]
mod tests {
    use redis::RedisResult;
    use crate::models::models::RedisORM;
    use crate::open_food_facts;
    use crate::models::product::Product;

    #[tokio::test]
    async fn test_cached_lowercase() {
        let product_name ="Test Köllnflocken Blütenzart";

        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        let mut test_product = Product::example();
        test_product.code = "123".to_string();
        test_product.product_name = Some(product_name.to_string());
        test_product.save(&mut con).expect("DIDNT SAVE");
        let prod = Product::fetch_from_uuid(&mut con, &test_product.code).expect("DIDNT FETCH");
        assert!(prod.product_name.is_some());
        dbg!(prod.clone());

        let res : Vec<Product> = Product::search_local(&mut con, &product_name);
        dbg!(&res);
        let binding = res.first().unwrap();
        assert_eq!(binding.code,test_product.code);
    }
    #[tokio::test]
    async fn test_cached_query() {
        let product_name ="FLOCKEN Kölln";

        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        let mut test_product = Product::example();
        test_product.code = "123".to_string();
        test_product.save(&mut con).expect("DIDNT SAVE");

        let res: RedisResult<Option<String>> = redis::cmd("GET").arg("product_name:".to_string()+product_name).query(&mut con);
        dbg!(&res);
        assert_eq!(res, Ok(Some(test_product.code)));
    }
    #[tokio::test]
    async fn test_cached_query_real_search() {
        let product_name ="Nutella";

        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        let mut test_product = open_food_facts::sdk::search_openff(product_name).await.unwrap().first().unwrap().clone();
        dbg!(test_product.clone());
        test_product.save(&mut con).expect("DIDNT SAVE");

        let res: RedisResult<Option<Vec<String>>> = redis::cmd("KEYS").arg("product_name:*".to_string()+product_name+"*").query(&mut con);
        let binding = res.unwrap().unwrap();
        let fetched_code = binding.first().unwrap();
        let res: RedisResult<Option<String>> = redis::cmd("GET").arg(fetched_code).query(&mut con);
        dbg!(&res);
        let binding = res.unwrap().unwrap();
        assert_eq!(test_product.code,binding);
    }
    #[tokio::test]
    async fn test_save_multi() {
        let product_name ="Nutella";

        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        let mut test_products = open_food_facts::sdk::search_openff(product_name).await.unwrap();
        for test_product in test_products.iter(){
            test_product.save(&mut con).expect("DIDNT SAVE");
        }

        let res: RedisResult<Option<Vec<String>>> = redis::cmd("KEYS").arg("product_name:*".to_string()+product_name+"*").query(&mut con);
        let binding = res.unwrap().unwrap();
        dbg!(binding.clone());
        for fetched_code in binding.iter(){
            let res: RedisResult<Option<String>> = redis::cmd("GET").arg(fetched_code).query(&mut con);
            dbg!(&res);
            let binding = res.unwrap().unwrap();
            assert!(test_products.iter().any(|x| x.code == binding));
        }

    }
}