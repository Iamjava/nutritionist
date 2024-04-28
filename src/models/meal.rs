use std::fmt::{Debug, Error};
use std::ops::{Add, Mul};
use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::models::RedisORM;
use crate::{db, models, open_food_facts};
use crate::db::connector::{default_fetch_from_uuid, default_save};
use crate::models::product::Product;
use crate::open_food_facts::models::{Nutriments, OpenFFValue};

#[derive(Debug, Serialize, Deserialize,Clone)]
pub(crate) struct Meal {
    pub(crate) contents: Vec<MealContent>,
    pub(crate) id: Uuid,
    pub(crate) user_id: String,
    pub(crate) date: chrono::DateTime<chrono::Utc>,
}


impl Add for OpenFFValue {
    type Output = OpenFFValue;

    fn add(self, rhs: Self) -> Self::Output {
        OpenFFValue::Flt(self.to_numerical() + rhs.to_numerical())
    }
}

impl Add for Nutriments {
    type Output = Nutriments;

    fn add(self, rhs: Self) -> Self::Output {
        Nutriments {
            carbohydrates_100g: Some(self.carbohydrates_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.carbohydrates_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            sugars_100g: Some(self.sugars_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.sugars_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            proteins_100g: Some(self.proteins_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.proteins_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            fat_100g: Some(self.fat_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.fat_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            energy_kcal_100g: Some(self.energy_kcal_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.energy_kcal_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            fiber_100g: Some(self.fiber_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.fiber_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            salt_100g: Some(self.salt_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.salt_100g.unwrap_or(OpenFFValue::Flt(0.0))),
            sodium_100g: Some(self.sodium_100g.unwrap_or(OpenFFValue::Flt(0.0)) + rhs.sodium_100g.unwrap_or(OpenFFValue::Flt(0.0))),
        }

    }
}

impl Meal {
    pub(crate) fn get_macros(&self) -> Nutriments {
        let mut nutriments = Nutriments::default();
        let prods = self.contents.clone();
        for product in prods.into_iter() {
                let mut i = product.product.get_numerical_macros();
                nutriments = nutriments + i;
        }
        nutriments
    }

    pub(crate) fn get_kcal(&self) -> Result<f32,Error> {
        let mut kcal = 0.0;
        let prods = self.contents.clone();
        for product in prods.into_iter() {
            if let Some(nutriments) = product.product.nutriments {
                let e:OpenFFValue = nutriments.energy_kcal_100g.unwrap_or(OpenFFValue::Flt(0f32));
                kcal += <OpenFFValue as TryInto<f32>>::try_into(e).expect("TODO: panic message");
            }
        }
        Ok(kcal)
    }

    fn add_meal_to_user(&self, con: &mut Connection,user_id: &str) {
     db::connector::add_to_set(con, "meals:".to_string()+ &*self.user_id, self.id.to_string()).unwrap()
    }
    fn remove_meal_from_user(&self, con: &mut Connection,user_id: &str) {
        db::connector::remove_from_set(con, "meals:".to_string()+ &*self.user_id, self.id.to_string()).unwrap()
    }
}
impl RedisORM for Meal {
    // Id is meal:user_id:meal_uuid
    fn save(&self, con: &mut Connection) -> RedisResult<()> where Self: Serialize {
        let id =self.user_id.to_string()+":"+ &*self.id.to_string();
        self.add_meal_to_user(con, &self.user_id);
        default_save(con, Self::redis_type_name().as_str(), &self.id.to_string(), self)
    }

    // Id is meal:user_id:meal_uuid
    fn example() -> Meal {
        Meal {
            contents: vec![],
            id: Uuid::new_v4(),
            user_id: "12345".to_string(),
            date: chrono::Utc::now(),
        }
    }

    fn redis_type_name() -> String {
        "meal".to_string()
    }

    fn redis_id(&self) -> String {
        self.id.to_string()
    }

}
// tokio tests
#[cfg(test)]
mod tests {
    use crate::models::models::RedisORM;
    use crate::models::user::User;
    use crate::{db, models};
    use crate::open_food_facts::sdk::search_openff;

    #[tokio::test]
    async fn test_meal() {
        let mut con = db::connector::get_connection().unwrap();
        let user = User::example();
        let user = User::check_if_exists_or_create(&mut con, &user.id, &user.name, &user.email).unwrap();

        let prod1 = search_openff("Kölln Müsli").await.unwrap();
        let prod2 = search_openff("Nutella").await.unwrap();

        let mut meal = models::meal::Meal::example();

        meal.contents.push(prod1.first().unwrap().clone().into());
        meal.contents.push(prod2.first().unwrap().clone().into());
        meal.save(&mut con).unwrap();

        let meal_fetched = models::meal::Meal::fetch_from_uuid(&mut con, &meal.id.to_string());
        dbg!(meal_fetched.clone());
        assert!(meal_fetched.is_some());
    }
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct MealContent {
    pub(crate) product: Product,
    pub(crate) quantity: f32,
    pub(crate) id: Uuid,
}
