use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use crate::db::connector::default_save_expire;
use crate::models::models::{NutritionistSearchQuery, RedisORM};
use crate::models::product::Product;


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchResult {
    pub(crate) products: Vec<Product>,
    #[serde(skip)]
    pub(crate) query: NutritionistSearchQuery,
}


impl RedisORM for  SearchResult{
    fn save(&self, con: &mut Connection) -> RedisResult<()> {
        println!("Saving search result {:?}", self.query.query);
        default_save_expire(con, "ffsearch", &self.query.query, self, 60 * 60 * 24 * 7)
    }

    fn example() -> Self where Self: Sized {
        SearchResult {
            products: vec![],
            query: NutritionistSearchQuery {
                query: "".to_string(),
            }
        }
    }


    fn redis_type_name() -> String {
        "ffsearch".to_string()
    }

    fn redis_id(&self) -> String {
        self.query.query.clone()
    }
}
pub async fn cached_search(search: impl Into<OpenFoodFactsQuery>) -> Result<Vec<Product>, reqwest::Error> {
    let search = search.into();
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let cache = SearchResult::fetch_from_uuid(&mut con, &search.search_query);

    if cache.is_some() {
        return Ok(cache.unwrap().products)
    }
    let result =  search_openff(search.clone()).await?;
    // Shit Code, DONT do it like that
    let bind = result.clone();
    for product in bind.iter() {
        product.save(&mut con).expect("Could not save product");
    }
    let search_result = SearchResult {
        products: result.clone(),
        query: NutritionistSearchQuery {
            query: search.search_query,
        }
    };
    search_result.save(&mut con).expect("Could not save search result");
    return Ok(result)
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
    let response = dbg!(client.get(url).query(&params)).send().await?;
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
