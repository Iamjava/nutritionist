extern crate reqwest;

use serde::{Deserialize, Serialize};
use crate::models::models::RedisORM;
use crate::open_food_facts::models::OpenFFValue::{Flt, Str};

#[derive(Serialize, Deserialize,Clone, Debug)]
#[serde(untagged)]
pub enum OpenFFValue{
    Str(String),
    Flt(f32),
    None,
}

impl OpenFFValue{
    pub fn to_numerical(&self) -> f32 {
        match self {
            Flt(x) => *x,
            Str(s) => s.parse().unwrap_or(0.0),
            _=> 0.0
        }
    }
}


impl TryInto<f32> for OpenFFValue{
    type Error = String;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Flt(x) => Ok(x),
            Str(s) =>  s.parse().map(|num| num).or(Err(format!("OpenFF cant parse value {:?}",s))),
            _=> Err("OpenFFValue is None".to_string())
        }
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nutriments{
    pub(crate) carbohydrates_100g: Option<OpenFFValue>,
    pub(crate) sugars_100g: Option<OpenFFValue>,
    pub(crate) proteins_100g: Option<OpenFFValue>,
    pub(crate) fat_100g: Option<OpenFFValue>,
    #[serde(alias = "energy-kcal_100g")]
    pub(crate) energy_kcal_100g: Option<OpenFFValue>,
    pub(crate) fiber_100g: Option<OpenFFValue>,
    pub(crate) salt_100g: Option<OpenFFValue>,
    pub(crate) sodium_100g: Option<OpenFFValue>,
}
impl Default for Nutriments {
    fn default() -> Self {
        Nutriments {
            carbohydrates_100g: Some(OpenFFValue::Flt(0.0)),
            sugars_100g: Some(OpenFFValue::Flt(0.0)),
            proteins_100g: Some(OpenFFValue::Flt(0.0)),
            fat_100g: Some(OpenFFValue::Flt(0.0)),
            energy_kcal_100g: Some(OpenFFValue::Flt(0.0)),
            fiber_100g: Some(OpenFFValue::Flt(0.0)),
            salt_100g: Some(OpenFFValue::Flt(0.0)),
            sodium_100g: Some(OpenFFValue::Flt(0.0)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

