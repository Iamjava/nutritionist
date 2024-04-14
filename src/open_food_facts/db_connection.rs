use redis::{Connection, RedisResult};
use uuid::Uuid;
use crate::db::connector::{default_fetch_and_parse, default_fetch_all, default_fetch_from_uuid, default_save};
use crate::models::models::{Meal, RedisORM};
use crate::open_food_facts::models::Product;

impl RedisORM for Product{
    fn save(&self, con: &mut Connection) -> RedisResult<()> {
        default_save(con, "product", &self.code, &self)?;
        let product_name= self.product_name.clone();
        let val = &*product_name.clone().unwrap().to_lowercase();
        dbg!(val.clone());
        if product_name.is_some(){
            let result = redis::cmd("SET").arg("product_name:".to_string()+ val).arg(&self.code).query(con);
            return result
        }
        return Ok(())
    }

    fn fetch_from_uuid(con: &mut Connection, id: &str) -> Option<Self> where Self: Sized {
        default_fetch_from_uuid(con,"product",id)
    }

    fn default() -> Self where Self: Sized {
        Product{
            code: "".to_string(),
            nutrition_grades: None,
            product_name: None,
            nutriments: None,
        }
    }

    fn all(con: &mut Connection) -> Vec<Self> where Self: Sized {
        default_fetch_all(con,"product")
    }
}

impl RedisORM for Meal {
    fn save(&self,con: & mut redis::Connection) -> redis::RedisResult<()> {
        default_save(con, "meal", &self.id.to_string(), self)
    }

    fn fetch_from_uuid(con: & mut redis::Connection, id: &str,) -> Option<Meal>{
        default_fetch_from_uuid(con, "meal", id)
    }

    fn default() -> Meal {
        Meal {
            contents: vec![],
            id: Uuid::new_v4(),
        }
    }

    fn all(con: &mut Connection) -> Vec<Self> where Self: Sized {
        default_fetch_all(con, "meal")
    }
}

#[cfg(test)]
mod tests {
    use redis::RedisResult;
    use crate::models::models::RedisORM;
    use crate::open_food_facts;
    use crate::open_food_facts::models::Product;

    #[tokio::test]
    async fn test_cached_lowercase() {
        let product_name ="Test Köllnflocken Blütenzart";

        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        let mut test_product = Product::default();
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
        let mut test_product = Product::default();
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
        let mut test_product = open_food_facts::sdk::search_openff(product_name).await.unwrap().products.first().unwrap().clone();
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
        let mut test_products = open_food_facts::sdk::search_openff(product_name).await.unwrap().products;
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