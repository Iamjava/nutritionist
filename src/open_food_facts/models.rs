extern crate reqwest;

use redis::Connection;
use serde::{Deserialize, Deserializer, Serialize};
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

    #[serde(deserialize_with = "deserialize_string_or_float")]
    pub(crate) carbohydrates_100g: Option<f32>,
    #[serde(deserialize_with = "deserialize_string_or_float")]
    pub(crate) sugars: Option<f32>,
    #[serde(deserialize_with = "deserialize_string_or_float")]
    pub(crate) proteins_100g: Option<f32>,
    #[serde(deserialize_with = "deserialize_string_or_float")]
    pub(crate) fat_100g: Option<f32>,
    #[serde(alias = "energy-kcal_100g")]
    #[serde(deserialize_with = "deserialize_string_or_float")]
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



fn deserialize_string_or_float<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
    where
        D: Deserializer<'de>,
{
    // Deserialize the value as a serde_json::Value first
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(n) => {
            // If the value is a number, deserialize it as a f32
            if let Some(number) = n.as_f64() {
                Ok(Some(number as f32))
            } else {
                Err(serde::de::Error::custom("Invalid number format"))
            }
        }
        serde_json::Value::String(s) => {
            // If the value is a string, try parsing it as a float
            s.parse().map(|num| Some(num)).map_err(serde::de::Error::custom)
        }
        _ => Err(serde::de::Error::custom("Invalid value type")),
    }
}

#[cfg(test)]
mod test_models{

}

