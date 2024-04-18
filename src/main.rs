mod open_food_facts;
mod db;
mod models;
mod app;

#[tokio::main]
async fn main() {
    app::server::serve().await;
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::models::models;
    use uuid::Uuid;
    use crate::models::models::RedisORM;
    use crate::db::connector;
    use crate::open_food_facts;
    use crate::open_food_facts::models::Product;


    #[tokio::test]
    async fn test_serialisation(){
        let test_meal = models::Meal {
            contents: vec![],
            id: Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e9").unwrap(),
        };
        let mut con = connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        test_meal.save(&mut con).expect("DIDNT SAVE");

        let meal = models::Meal::fetch_from_uuid(&mut con, &Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e9").unwrap().to_string());
        assert!(meal.is_some());
        let meal = models::Meal::fetch_from_uuid(&mut con, &Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e8").unwrap().to_string());
        assert!(meal.is_none());
    }


    #[tokio::test]
    async fn create_test_data(){
        let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");

        let mut test_product = Product::default();
        test_product.code = "123".to_string();
        test_product.product_name = Some("Test Product1".to_string());
        test_product.save(&mut con).unwrap();

        let mut test_product = Product::default();
        test_product.code = "124".to_string();
        test_product.product_name = Some("Test Productw".to_string());
        test_product.save(&mut con).unwrap();

        let mut test_product = Product::default();
        test_product.code = "125".to_string();
        test_product.product_name = Some("Test Product3".to_string());
        test_product.save(&mut con).unwrap();
    }
    #[tokio::test]
    async fn test_all_meals(){
        let test_meal = models::Meal {
            contents: vec![],
            id: Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e7").unwrap(),
        };
        let mut con = connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
        test_meal.save(&mut con).expect("DIDNT SAVE");

        let meals = models::Meal::all(&mut con);
        println!("{:?}", meals);
        assert!(meals.len()>= 1);
    }

    #[tokio::test]
    async fn test_all_products(){
        let mut con = connector::get_connection().expect("Could not connect to redis,maybe redis is not running");

        let test_product = open_food_facts::sdk::search_openff("Kölln Flocken").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");
        let test_product = open_food_facts::sdk::search_openff("Nutella").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");

        let test_product = open_food_facts::sdk::search_openff("ja lasagne").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");

        let test_product = open_food_facts::sdk::search_openff("lasagne").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");
        let test_product = open_food_facts::sdk::search_openff("Brot").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");
        let test_product = open_food_facts::sdk::search_openff("Butter").await.unwrap().first().unwrap().clone();
        test_product.save(&mut con).expect("DIDNT SAVE");
        let products = open_food_facts::models::Product::all(&mut con);
        println!("{:?}", products);
        assert!(products.len()>= 1);
    }

}
