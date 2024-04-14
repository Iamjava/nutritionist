use std::convert::Infallible;
use std::fs;
use std::io::read_to_string;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::Router;
use axum::routing::{get, post};
use crate::models::models::{NutritionistSearchQuery, RedisORM};
use crate::open_food_facts::models::Product;
use crate::open_food_facts::sdk::search_openff;

pub async fn serve(){
             let app = Router::new()
            .route("/:query", get(|key: Path<String>| async move {
                    let prods: Vec<_> = search_openff(key.0).await.unwrap().products.iter().map(|x| format!("{} {:?}", &x.product_name.clone().unwrap(), &x.nutriments.clone().unwrap().energy_kcal_100g)).collect();

                    format!("Hello, World! {:?}", prods)
            }))
            .route("/search", post(|x: axum::Form<NutritionistSearchQuery>| async {  handle_search_post(x).await }))
            .route("/search", get(|| async { handle_search().await }))
            .route("/test", post(|| async { handle_search_test().await }));

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
}

fn to_html_search_item(item: Product) -> String {
    format!("<h3>{}</h1>", item.product_name.unwrap())
}

async fn handle_search_post(x: axum::Form<NutritionistSearchQuery>) -> Response<String> {
    let product = search_openff(&*x.query).await.unwrap().products;
    let prods :Vec<_> = product.iter().filter(|p| p.product_name.is_some()).map(|x| to_html_search_item(x.clone())).collect();
    let string_from_template = format!("<h1>Search Results for {}</h1>{}", x.query, prods.join(""));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}
async fn handle_search_test() -> Response<String> {
    let string_from_template =  "TEST RESPONSE";

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}
async fn handle_search() -> Response<String> {
    let string_from_template =fs::read_to_string("templates/test/search.html").unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}