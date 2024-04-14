use crate::open_food_facts::models::{OpenFoodFactsQuery, SearchResult};


pub async fn search_openff(search: impl Into<OpenFoodFactsQuery>) -> Result<SearchResult, reqwest::Error> {
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
    let search_result: SearchResult = response.json().await?;
    Ok(search_result)
}
