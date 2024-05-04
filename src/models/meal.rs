use std::fmt::{Debug, Error};
use crate::db::connector;
use std::ops::{Add, Mul};
use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db;
use crate::models::models::RedisORM;
use crate::db::connector::{default_fetch_from_uuid, default_save};
use crate::usda::search::{Food, NutrientValues};

#[derive(Debug, Serialize, Deserialize,Clone)]
pub(crate) struct Meal {
    pub(crate) contents: Vec<MealContent>,
    pub(crate) id: Uuid,
    pub(crate) user_id: String,
    pub(crate) date: chrono::DateTime<chrono::Utc>,
}



impl Add for NutrientValues {
    type Output = NutrientValues;

    fn add(self, rhs: Self) -> Self::Output {
        NutrientValues{
            carbohydrates: self.carbohydrates + rhs.carbohydrates,
            proteins: self.proteins + rhs.proteins,
            fats: self.fats + rhs.fats,
            energy: self.energy + rhs.energy,
            fiber: self.fiber + rhs.fiber,
            salt: self.salt + rhs.salt,
            sodium: self.sodium + rhs.sodium,
            sugar: self.sugar + rhs.sugar,
        }
    }
}

impl Meal {
    pub(crate) fn get_macros(&self) -> NutrientValues {
        let mut nutriments = NutrientValues::default();
        let prods = self.contents.clone();
        for content in prods.into_iter() {
                let mut i = content.product.get_numerical_macros()* content.quantity*0.01;
                nutriments = nutriments + i;
        }
        nutriments
    }

    pub(crate) fn get_kcal(&self) -> Result<f32,Error> {
        let mut kcal = 0.0;
        let prods = self.contents.clone();
        for product in prods.into_iter() {
            let prod = product.product.clone().get_numerical_macros();
            kcal += prod.energy;
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

    #[tokio::test]
    async fn test_meal() {
    }
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct MealContent {
    pub(crate) product: Food,
    pub(crate) quantity: f32,
    pub(crate) id: Uuid,
}
