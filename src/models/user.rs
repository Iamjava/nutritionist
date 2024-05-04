use redis::{Connection, RedisError, RedisResult};
use serde::{Deserialize, Serialize};
use crate::db;
use crate::models::meal::Meal;
use crate::models::models::RedisORM;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct User{
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
}

impl RedisORM for User{

    fn example() -> Self where Self: Sized {
        User{
            id: "12345".to_string(),
            name: "TEST".to_string(),
            email: "test@test.de".to_string(),
        }
    }

    fn redis_type_name() -> String {
        "user".to_string()
    }

    fn redis_id(&self) -> String {
        self.id.clone()
    }
}

impl User {
    pub(crate) fn check_if_exists_or_create(con: &mut Connection, id: &str, name: &str, email: &str) -> Result<User, RedisError> {
        let user = User::fetch_from_uuid(con, id);
        if user.is_some() {
            return Ok(user.unwrap());
        }
        let user = User{
            id: id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
        };
        user.save(con)?;
        return Ok(user);
    }
    fn create_default_user(con: &mut Connection) -> Result<User, RedisError> {
        let user = User::example();
        user.save(con)?;
        return Ok(user);
    }
    pub(crate) fn fetch_user_meals(&self, con: &mut Connection) -> Vec<Meal> {
        let keys: Vec<String> = db::connector::get_set(con, "meals:".to_string()+ &*self.id);
        dbg!(keys.clone());
        let mut meals: Vec<Meal> = vec![];
        for key in keys.iter() {
            let meal: Meal = Meal::fetch_from_uuid(con, key).unwrap();
            dbg!(meal.clone());
            meals.push(meal);
        }
        meals
    }
}

#[cfg(test)]
mod tests {
    use crate::models::models::RedisORM;
    use crate::models::user::User;
    use redis::Commands;
    use crate::{db, models};
    use crate::models::meal::Meal;

    #[test]
    fn test_user() {
        let mut con = db::connector::get_connection().unwrap();
        let user = User::example();
        let user = User::check_if_exists_or_create(&mut con, &user.id, &user.name, &user.email).unwrap();

        let user2 = User::fetch_from_uuid(&mut con, &user.id);
        assert!(user2.is_some());
    }
    #[tokio::test]
    async fn test_user_meals() {
    }
}