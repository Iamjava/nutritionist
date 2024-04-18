use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use crate::db::connector::{default_fetch_all, default_fetch_from_uuid, default_save};
use crate::models::models::{NutritionistSearchQuery, RedisORM};
use crate::open_food_facts::models::{OpenFoodFactsQuery, Product,};


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchResult {
    pub(crate) products: Vec<Product>,
    #[serde(skip)]
    pub(crate) query: NutritionistSearchQuery,
}


impl RedisORM for  SearchResult{
    fn save(&self, con: &mut Connection) -> RedisResult<()> {
        println!("Saving search result {:?}", self.query.query);
        default_save(con, "ffsearch", &self.query.query, self)
    }

    fn fetch_from_uuid(con: &mut Connection, id: &str) -> Option<Self> where Self: Sized {
      default_fetch_from_uuid(con, "ffsearch", id)
    }

    fn default() -> Self where Self: Sized {
        SearchResult {
            products: vec![],
            query: NutritionistSearchQuery {
                query: "".to_string(),
            }
        }
    }

    fn all(con: &mut Connection) -> Vec<Self> where Self: Sized {
        default_fetch_all(con, "ffsearch")
    }
}
pub async fn cached_search(search: impl Into<OpenFoodFactsQuery>) -> Result<Vec<Product>, reqwest::Error> {
    let search = search.into();
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let cache = SearchResult::fetch_from_uuid(&mut con, &search.search_query);

    if cache.is_some() {
        return Ok(cache.unwrap().products)
    }
    return search_openff(search).await
}


pub async fn search_openff(search: impl Into<OpenFoodFactsQuery>) -> Result<Vec<Product>, reqwest::Error> {
    let url = "https://world.openfoodfacts.org/cgi/search.pl";
    let search = search.into();

    let params = [
        ("search_terms", &*search.search_query),
        ("search_simple", "1"),
        // add tags to the search
        ("action", "process"),
        ("json", "1"),
        ("fields", "code,nutrition_grades,categories_tags_en,product_name,nutriments"),
    ];
    let client = reqwest::Client::new();
    let response = client.get(url).query(&params).send().await?;
    let mut search_result: SearchResult = response.json().await?;
    search_result.query = NutritionistSearchQuery {
        query: search.search_query,
    };
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    search_result.save(&mut con).expect("Could not save search result");
    let prods = search_result.products.into_iter().filter(|x| {
        x.product_name.clone().is_some() && x.nutriments.is_some() && x.product_name.clone().unwrap() != ""
    }).collect();
    Ok(prods)
}
