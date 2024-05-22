use std::cmp::PartialEq;
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use crate::db;
use crate::models::meal::Meal;
use crate::models::models::RedisORM;
use redis::{Connection, RedisError, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) user_type: UserType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserType{
    User,
    Nutritionist(Vec<String>),
    Admin
}

impl RedisORM for User {
    fn example() -> Self
    where
        Self: Sized,
    {
        User {
            id: "12345".to_string(),
            name: "TEST".to_string(),
            email: "test@test.de".to_string(),
            user_type: UserType::User,
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

    pub(crate) fn check_if_exists_by_id(
        con: &mut Connection,
        id: &str,
    ) -> Result<User, String> {
        let user = User::fetch_from_uuid(con, id);
        let name= id.clone();
        let email = "test@test.de";
        if user.is_some() {
            let mut user = user.unwrap();
            return Ok(user);
        }
        return Err("User not found".into());
    }
    pub(crate) fn check_if_exists_or_create(
        con: &mut Connection,
        oidc_claims: OidcClaims<EmptyAdditionalClaims>
    ) -> Result<User, RedisError> {
        let id = oidc_claims.preferred_username().expect("username broken");
        let user = User::fetch_from_uuid(con, id);
        let name= id.clone();
        let email = "test@test.de";
        dbg!(name.to_string(),name.to_string() == "jan");
        let user_type = if name.to_string() == "jan" {
            UserType::Nutritionist(vec!["jan".to_string()])
        } else {
            UserType::User
        };
        if user.is_some() {
            let mut user = user.unwrap();
            user.user_type = user_type;
            return Ok(user);
        }
        let user = User {
            id: id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            user_type,
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
        let keys: Vec<String> = db::connector::get_set(con, "meals:".to_string() + &*self.id);
        let mut meals: Vec<Meal> = vec![];
        for key in keys.iter() {
            let meal: Meal = Meal::fetch_from_uuid(con, key).unwrap();
            meals.push(meal);
        }
        meals
    }
}

#[cfg(test)]
mod tests {
    use crate::models::meal::Meal;
    use crate::models::models::RedisORM;
    use crate::models::user::User;
    use crate::{db, models};
    use redis::Commands;

    #[test]
    fn test_user() {
    }
    #[tokio::test]
    async fn test_user_meals() {}
}
