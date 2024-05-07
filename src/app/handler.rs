use crate::db;
use crate::models::models::{NutritionistSearchQuery, RedisORM};
use crate::usda::search::cached_search;
use crate::usda::search::Food; // bring trait in scope
use askama::Template;
use axum::extract::Path;
use axum::http::{Response, StatusCode};

#[derive(Template)] // this will generate the code...
#[template(path = "product/search_response_food.html")] // using the template in this path, relative
                                                        // to the `templates` dir in the crate root
struct SearchResponseTemplate<'a> {
    // the name of the struct can be anything
    meal_id: &'a str,
    foods: Vec<Food>,
}

pub async fn search_usda_handler(
    Path(id): Path<String>,
    x: axum::Form<NutritionistSearchQuery>,
) -> Response<String> {
    let result = cached_search(&*x.query).await;
    let search_response = SearchResponseTemplate {
        meal_id: &*id,
        foods: result,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(search_response.render().unwrap())
        .unwrap()
}
