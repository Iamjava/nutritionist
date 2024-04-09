extern crate reqwest;

use redis::Connection;
use serde::{Deserialize, Serialize};
use crate::db::connector::{default_fetch_all, default_fetch_from_uuid, default_save};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub(crate) code: String,
    pub(crate) nutrition_grades: Option<String>,
    pub(crate) product_name: Option<String>,
    pub(crate) nutriments: Option<Nutriments>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum Weirdness {
    S(String),
    F(f32),
}


impl crate::models::models::RedisInterface for Product {
    fn save(&self, con: &mut redis::Connection) -> redis::RedisResult<()> {
        default_save(con, "product", &self.code, self)
    }

    fn fetch_from_uuid(con: &mut redis::Connection, id: &str) -> Option<Product> {
        default_fetch_from_uuid(con, "product", id)
    }

    fn new() -> Product {
        Product {
            code: "".to_string(),
            nutrition_grades: None,
            product_name: None,
            nutriments: None,
        }
    }

    fn all(con: &mut Connection) -> Vec<Self> where Self: Sized {
        default_fetch_all(con, "product")

    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nutriments{
    pub(crate) carbohydrates_100g: Option<f32>,
    pub(crate) sugars: Option<f32>,
    pub(crate) proteins_100g: Option<f32>,
    pub(crate) fat_100g: Option<f32>,
    #[serde(alias = "energy-kcal_100g")]
    pub(crate) energy_kcal_100g: Option<f32>,
}
impl Nutriments {
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub(crate) products: Vec<Product>,
}

pub(crate) struct OpenFoodFactsQuery {
    pub(crate) search_query: String,
    pub(crate) tags: Vec<String>,
}

impl Into<OpenFoodFactsQuery> for &str {
    fn into(self) -> OpenFoodFactsQuery {
        OpenFoodFactsQuery::new(self.to_string())
    }
}
impl Into<OpenFoodFactsQuery> for String {
    fn into(self) -> OpenFoodFactsQuery {
        OpenFoodFactsQuery::new(self)
    }
}

impl OpenFoodFactsQuery {
    pub(crate) fn new(search_query: String) -> OpenFoodFactsQuery {
             OpenFoodFactsQuery {
            search_query,
            tags: vec!["de".to_string()],
        }
    }
}

#[cfg(test)]
mod test_models{

}

