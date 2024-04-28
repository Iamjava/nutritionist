use std::fmt::Debug;
use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::connector::{default_fetch, default_fetch_all, default_save};
use crate::models::meal::MealContent;
use crate::models::models::RedisORM;
use crate::open_food_facts::models::{Nutriments, OpenFFValue};

#[derive( Serialize, Deserialize, Clone)]
pub struct Product {
    pub(crate) code: String,
    pub(crate) nutrition_grades: Option<String>,
    pub(crate) product_name: Option<String>,
    pub(crate) nutriments: Option<Nutriments>,
}

impl Into<MealContent> for Product{
    fn into(self) -> MealContent {
        MealContent {
            product: self,
            quantity: 1.0,
            id: Uuid::new_v4(),
        }
    }

}

impl Product {
    pub(crate) fn search_local(con: &mut Connection, query: &str) -> Vec<Product> {
        let product_name = query.to_lowercase();
        let keys: Vec<String> = redis::Cmd::keys("product_name:*".to_string() + &product_name +"*").query(con).unwrap();
        let mut products: Vec<Product> = vec![];
        for key in keys.iter() {
            let product_id =  default_fetch(con, key).unwrap();
            let product = Product::fetch_from_uuid(con, &product_id.unwrap());

            dbg!(product.clone());
            if product.is_some() {
                products.push(product.unwrap());
            }
        }
        products
    }
    pub(crate) fn get_numerical_macros(&self) -> Nutriments {
        let nutriments = self.nutriments.clone().unwrap();
        let protein = nutriments.proteins_100g.unwrap_or(OpenFFValue::None);
        let fat = nutriments.fat_100g.unwrap_or(OpenFFValue::None);
        let carbs = nutriments.carbohydrates_100g.unwrap_or(OpenFFValue::None);
        let kcal = nutriments.energy_kcal_100g.unwrap_or(OpenFFValue::None);
        let fiber = nutriments.fiber_100g.unwrap_or(OpenFFValue::None);
        let salt = nutriments.salt_100g.unwrap_or(OpenFFValue::None);
        let sodium = nutriments.sodium_100g.unwrap_or(OpenFFValue::None);
        let sugar = nutriments.sugars_100g.unwrap_or(OpenFFValue::None);

        Nutriments{
            carbohydrates_100g: Some(carbs),
            sugars_100g: Some(sugar),
            proteins_100g: Some(protein),
            fat_100g: Some(fat),
            energy_kcal_100g: Some(kcal),
            fiber_100g: Some(fiber),
            salt_100g: Some(salt),
            sodium_100g: Some(sodium),
        }
    }
}
impl Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Product")
            .field("product_name", &self.product_name)
            .field("macros", &self.get_numerical_macros())
            .finish()
    }
}

impl RedisORM for Product{
    fn save(&self, con: &mut Connection) -> RedisResult<()> {
        default_save(con, "product", &self.code, &self)?;
        let product_name= self.product_name.clone();
        let val = &*product_name.clone().unwrap().to_lowercase();
        if product_name.is_some(){
            let result = redis::cmd("SET").arg("product_name:".to_string()+ val).arg(&self.code).query(con);
            return result
        }
        return Ok(())
    }

    fn example() -> Self where Self: Sized {
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

    fn redis_type_name() -> String {
        "product".to_string()
    }

    fn redis_id(&self) -> String {
        self.code.clone()
    }
}


