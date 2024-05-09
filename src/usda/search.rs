use crate::db::connector::default_save_expire;
use crate::models::models::RedisORM;
use redis::Connection;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::ops::Mul;
use uuid::Uuid;

#[derive(Deserialize, Clone, Serialize, Debug)]
struct Nutrient {
    #[serde(rename = "nutrientId")]
    id: i32,
    #[serde(rename = "nutrientName")]
    name: String,
    #[serde(rename = "value")]
    amount: Option<f32>,
    #[serde(rename = "unitName")]
    unit: String,
    median: Option<f32>,
}

impl Default for Nutrient {
    fn default() -> Self {
        Nutrient {
            id: 0,
            name: "".to_string(),
            amount: Some(0.0),
            unit: "".to_string(),
            median: Some(0.0),
        }
    }
}

impl Mul<f32> for Food {
    type Output = Food;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut f = self.clone();
        f.nutrient_values = self.nutrient_values * rhs;
        f
    }
}
#[derive(Deserialize, Clone, Serialize)]
pub struct Food {
    pub description: String,
    #[serde(rename = "foodNutrients")]
    nutrients: Vec<Nutrient>,
    #[serde(rename = "fdcId")]
    pub id: i32,
    #[serde(skip)]
    pub nutrient_values: NutrientValues,
}
impl Debug for Food {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Food")
            .field("description", &self.description)
            .field("id", &self.id)
            .field("nutrients_values", &self.nutrient_values)
            .finish()
    }
}

impl Food {
    pub fn generate_nutrient_values(self) -> Self {
        Food {
            nutrient_values: self.get_numerical_macros(),
            ..self
        }
    }
    pub fn new(id: i32, description: impl Into<String>, nutrients: Vec<Nutrient>) -> Self {
        let mut food = Food {
            description: description.into(),
            nutrients,
            id,
            nutrient_values: NutrientValues::default(),
        };
        food = food.generate_nutrient_values();
        food
    }
}
impl RedisORM for Food {
    fn example() -> Self
    where
        Self: Sized,
    {
        Food {
            description: "".to_string(),
            nutrients: vec![],
            id: 1,
            nutrient_values: NutrientValues::default(),
        }
    }

    fn redis_type_name() -> String {
        "usda_food".to_string()
    }

    fn redis_id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct NutrientValues {
    pub carbohydrates: f32,
    pub proteins: f32,
    pub fats: f32,
    pub energy: f32,
    pub fiber: f32,
    pub salt: f32,
    pub sodium: f32,
    pub sugar: f32,
}
impl Default for NutrientValues {
    fn default() -> Self {
        NutrientValues {
            carbohydrates: 0.0,
            proteins: 0.0,
            fats: 0.0,
            energy: 0.0,
            fiber: 0.0,
            salt: 0.0,
            sodium: 0.0,
            sugar: 0.0,
        }
    }
}

impl Mul<f32> for NutrientValues {
    type Output = NutrientValues;

    fn mul(self, rhs: f32) -> Self::Output {
        NutrientValues {
            carbohydrates: self.carbohydrates * rhs,
            proteins: self.proteins * rhs,
            fats: self.fats * rhs,
            energy: self.energy * rhs,
            fiber: self.fiber * rhs,
            salt: self.salt * rhs,
            sodium: self.sodium * rhs,
            sugar: self.sugar * rhs,
        }
    }
}

impl Food {
    pub fn get_numerical_macros(&self) -> NutrientValues {
        let nuts = self.nutrients.clone();
        let default_nut = Nutrient::default();
        // let energy either ENERGY or Energy (Atwater General Factors)

        NutrientValues {
            carbohydrates: nuts
                .iter()
                .find(|x| x.name == "Carbohydrate, by difference")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
            proteins: nuts
                .iter()
                .find(|x| x.name == "Protein")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
            fats: nuts
                .iter()
                .find(|x| x.name == "Total lipid (fat)")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
            energy: nuts
                .iter()
                .find(|x| x.name == "Energy" || x.name == "Energy (Atwater Specific Factors)")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
            fiber: nuts
                .iter()
                .find(|x| x.name == "Fiber, total dietary")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
            sodium: nuts
                .iter()
                .find(|x| x.name == "Sodium, Na")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0) as f32
                * 0.001,
            salt: nuts
                .iter()
                .find(|x| x.name == "Sodium, Na")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0) as f32
                * 2.5f32
                * 0.001,
            sugar: nuts
                .iter()
                .find(|x| x.name == "Total Sugars")
                .unwrap_or(&default_nut)
                .amount
                .unwrap_or(0.0),
        }
    }
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct SearchResult {
    foods: Vec<Food>,
}

pub(crate) async fn query_usda_food_database(
    search_term: &str,
) -> Result<Vec<Food>, Box<dyn Error>> {
    let mut con = crate::db::connector::get_connection().unwrap();
    let api_key = "DEMO_KEY"; // Replace 'DEMO_KEY' with your actual API key
                              //let url = format!("https://api.nal.usda.gov/fdc/v1/foods/search?query={}&dataType=Foundation,SR%20Legacy,Survey%20%28FNDDS%29,Branded&pageSize=30&pageNumber=1&sortBy=dataType.keyword&sortOrder=asc&api_key={}", search_term, api_key);
    let url = format!("https://api.nal.usda.gov/fdc/v1/foods/search?query={}&dataType=Foundation,SR%20Legacy&pageSize=30&pageNumber=1&sortBy=dataType.keyword&sortOrder=asc&api_key={}", search_term, api_key);
    let mut result = vec![];

    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let mut resp: SearchResult = response.json().await?;
        if !resp.foods.is_empty() {
            for food in resp.foods.iter_mut() {
                food.nutrient_values = food.get_numerical_macros();
            }
            save_foods(&mut con, &mut result, &mut resp);
        } else {
            println!("No foods found.");
        }
    } else {
        println!("Failed to query the USDA Food Database.");
    }
    Ok(result)
}
fn save_foods(mut con: &mut Connection, result: &mut Vec<Food>, resp: &mut SearchResult) {
    let resp = resp.clone();
    for food in resp.foods {
        food.save(&mut con).unwrap();
        result.push(food);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FoodSearchResult {
    pub(crate) products: Vec<Food>,
    #[serde(skip)]
    pub(crate) query: String,
}

impl RedisORM for FoodSearchResult {
    fn save(&self, con: &mut Connection) -> redis::RedisResult<()> {
        default_save_expire(con, "usda_search", &self.query, self, 60 * 60 * 24 * 7)
    }

    fn example() -> Self
    where
        Self: Sized,
    {
        FoodSearchResult {
            products: vec![],
            query: "".to_string(),
        }
    }

    fn redis_type_name() -> String {
        "usda_search".to_string()
    }

    fn redis_id(&self) -> String {
        self.query.clone()
    }
}

pub async fn cached_search(query: &str) -> Vec<Food> {
    let mut con = crate::db::connector::get_connection().unwrap();
    let cache = FoodSearchResult::fetch_from_uuid(&mut con, query);
    if cache.is_some() {
        dbg!("Cache hit");
        return cache.unwrap().products;
    }
    let result = query_usda_food_database(query).await.unwrap();
    let search_result = FoodSearchResult {
        products: result.clone(),
        query: query.to_string(),
    };
    search_result.save(&mut con).unwrap();
    result
}

#[cfg(test)]
mod tests {

    #![allow(dead_code)]
    use crate::models::models::RedisORM;
    use crate::models::user::User;
    use crate::usda::search::{query_usda_food_database, Food};
    use crate::{db, models};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_usda_search() {
        let foods = crate::usda::search::query_usda_food_database("apple")
            .await
            .unwrap();
        assert!(!foods.is_empty());
    }
    #[tokio::test]
    async fn create_test_data() {
        let mut con = db::connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");

        let mut test_product = Food::example();
        test_product.id = 1;
        test_product.description = "Test Product1".to_string();
        test_product.save(&mut con).unwrap();

        let mut test_product = Food::example();
        test_product.id = 2;
        test_product.description = "Test Product2".to_string();
        test_product.save(&mut con).unwrap();

        let mut test_product = Food::example();
        test_product.id = 3;
        test_product.description = "Test Product3".to_string();
        test_product.save(&mut con).unwrap();

        let a = Food::fetch_from_uuid(&mut con, "1").unwrap();
        let b = Food::fetch_from_uuid(&mut con, "2").unwrap();
        let c = Food::fetch_from_uuid(&mut con, "3").unwrap();
        assert_eq!(a.description, "Test Product1");
        assert_eq!(b.description, "Test Product2");
        assert_eq!(c.description, "Test Product3");
    }

    #[tokio::test]
    async fn meal_test() {
        let mut con = db::connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");
        let mut meal = models::meal::Meal::example();
        meal.id = Uuid::new_v4();
        meal.username = "12345".to_string();
        meal.save(&mut con).expect("DIDNT SAVE");

        let mut meal = models::meal::Meal::fetch_from_uuid(&mut con, &meal.id.to_string()).unwrap();
        assert_eq!(meal.username, "12345");
    }

    #[tokio::test]
    async fn food_test() {
        let mut con = db::connector::get_connection()
            .expect("Could not connect to redis,maybe redis is not running");
        let mut food = Food::example();
        food.id = 1;
        food.description = "Test Product".to_string();
        assert_eq!(food.id.to_string(), "1");
        food.save(&mut con).expect("DIDNT SAVE");
        let food = Food::fetch_from_uuid(&mut con, "1").unwrap();
        assert_eq!(food.description, "Test Product");
    }
    #[tokio::test]
    async fn search_test() {
        let search = query_usda_food_database("tomato").await.unwrap();
        assert!(search.len() > 0);
    }
}
