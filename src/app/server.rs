use std::fs;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::Router;
use axum::routing::{get, post, put};
use tera::Tera;
use crate::models::models::{Meal, NutritionistSearchQuery, RedisORM};
use crate::open_food_facts::models::Product;
use crate::open_food_facts::sdk::{cached_search, search_openff};

// static once cell for Templates with tera
static TEMPLATES: once_cell::sync::Lazy<Tera> = once_cell::sync::Lazy::new(|| {
    let tera = Tera::new("templates/**/*").unwrap();
    tera
});


pub async fn serve() {
    let port = std::env::var("NUT_PORT").unwrap_or("8000".to_string());
    let app = Router::new()
    .route("/:query", get(|key: Path<String>| async move {
        let prods: Vec<_> = search_openff(key.0).await.unwrap().iter().map(|x| format!("{} {:?}", &x.product_name.clone().unwrap(), &x.nutriments.clone().unwrap().energy_kcal_100g)).collect();

        format!("Hello, World! {:?}", prods)
    }))
        .route("/", get(|| async { Response::builder().status(StatusCode::SEE_OTHER).header("Location", "/search").body("".to_string()).unwrap() }))
        .route("/search", post(|x: axum::Form<NutritionistSearchQuery>| async { handle_search_post(x).await }))
        .route("/search", get(|| async { handle_search().await }))
        .route("/test", post(|| async { handle_search_test().await }))
        .route("/newmeal", get(|| async { handle_create_meal().await }))
        .route("/meal/:id", get(|id: Path<String>| async { handle_meal(id).await }))
        .route("/meal/:id/addcontent/:code", get(|id: Path<(String, String)>| async { handle_add_content_to_meal(id).await }))
    ;

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn to_html_search_item(item: Product) -> String {
    format!("<h3>{}, {:?} kcal / 100g</h3>", item.product_name.unwrap(), item.nutriments.unwrap().energy_kcal_100g)
}

async fn handle_search_post(x: axum::Form<NutritionistSearchQuery>) -> Response<String> {
    let product = cached_search(&*x.query).await.unwrap();
    let prods: Vec<_> = product.iter().filter(|p| p.product_name.is_some()).map(|x| to_html_search_item(x.clone())).collect();
    let string_from_template = format!("<h1>Search Results for {}</h1>{}", x.query, prods.join(""));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

async fn handle_search_test() -> Response<String> {
    let string_from_template = "TEST RESPONSE";

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

// add a content to a meal /meal/:id/addcontent
async fn handle_add_content_to_meal(Path((id, code)): Path<(String, String)>) -> Response<String> {
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let mut meal = Meal::fetch_from_uuid(&mut con, &id).unwrap_or(Meal::default());
    let prod = Product::fetch_from_uuid(&mut con, &code).unwrap_or(Product::default());
    meal.contents.append(&mut vec![prod]);
    meal.save(&mut con).expect("DIDNT SAVE");
    dbg!(meal.clone());

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", format!("/meal/{}", id))
        .body("".into())
        .unwrap()
}


// Redirect to a newly created meal /meal/:id
async fn handle_create_meal() -> Response<String> {
    let meal = Meal::default();
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    meal.save(&mut con).expect("DIDNT SAVE");
    // send a redirect to the newly created meal
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", format!("/meal/{}", meal.id))
        .body("".into())
        .unwrap()
}

// Display a meal
async fn handle_meal(Path(id): Path<String>) -> Response<String> {
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let meal = Meal::fetch_from_uuid(&mut con, &id);
    let meal = meal.unwrap_or(Meal::default());
    let string_from_template = format!("<h1>Meal {}, {:?}</h1>", meal.id, meal.contents);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

async fn handle_search() -> Response<String> {
    let string_from_template = fs::read_to_string("templates/test/search.html").unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}