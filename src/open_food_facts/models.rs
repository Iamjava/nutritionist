extern crate reqwest;

use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use crate::db::connector::{default_fetch, default_fetch_and_parse};
use crate::models::models::RedisORM;
use crate::open_food_facts::models::OpenFFValue::{Flt, Str};

#[derive(Serialize, Deserialize,Clone, Debug)]
#[serde(untagged)]
pub enum OpenFFValue{
    Str(String),
    Flt(f32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub(crate) code: String,
    pub(crate) nutrition_grades: Option<String>,
    pub(crate) product_name: Option<String>,
    pub(crate) nutriments: Option<Nutriments>,
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
}

impl TryInto<f32> for OpenFFValue{
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Flt(x) => Ok(x),
            Str(s) =>  s.parse().map(|num| num).or(Err(format!("OpenFF cant parse value {:?}",s)))
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nutriments{

    pub(crate) carbohydrates_100g: Option<OpenFFValue>,
    pub(crate) sugars: Option<OpenFFValue>,
    pub(crate) proteins_100g: Option<OpenFFValue>,
    pub(crate) fat_100g: Option<OpenFFValue>,
    #[serde(alias = "energy-kcal_100g")]
    pub(crate) energy_kcal_100g: Option<OpenFFValue>,
}
impl Nutriments {
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
    use super::*;
    #[test]
    fn test_openff_value(){
        let flt: OpenFFValue = Flt(1.0);
        let str: OpenFFValue = Str("1.0".to_string());
        let flt_res: f32 = flt.try_into().unwrap();
        let str_res: f32 = str.try_into().unwrap();
        assert_eq!(flt_res, 1.0);
        assert_eq!(str_res, 1.0);
    }

}

