mod app;
mod db;
mod models;
mod usda;

#[tokio::main]
async fn main() {
    app::server::serve().await;
}
#[cfg(test)]
mod tests {
    use crate::db::connector;
    use crate::models::meal;
    use crate::models::models::RedisORM;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_serialisation() {
        let test_meal = meal::Meal {
            contents: vec![],
            id: Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e9").unwrap(),
            user_id: "12345".to_string(),
            date: chrono::Utc::now(),
        };
        let mut con = connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");
        test_meal.save(&mut con).expect("DIDNT SAVE");

        let meal = meal::Meal::fetch_from_uuid(
            &mut con,
            &Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e9")
                .unwrap()
                .to_string(),
        );
        assert!(meal.is_some());
        let meal = meal::Meal::fetch_from_uuid(
            &mut con,
            &Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e8")
                .unwrap()
                .to_string(),
        );
        assert!(meal.is_none());
    }

    #[tokio::test]
    async fn create_test_data() {
        let mut con = crate::db::connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");
    }
    #[tokio::test]
    async fn test_all_meals() {
        let test_meal = meal::Meal {
            contents: vec![],
            id: Uuid::from_str("0ff3917f-14a6-4d82-8d40-4a96cc6fc5e7").unwrap(),
            user_id: "12345".to_string(),
            date: chrono::Utc::now(),
        };
        let mut con = connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");
        test_meal.save(&mut con).expect("DIDNT SAVE");

        let meals = meal::Meal::all(&mut con);
        println!("{:?}", meals);
        assert!(meals.len() >= 1);
    }

    #[tokio::test]
    async fn test_all_products() {}
}
