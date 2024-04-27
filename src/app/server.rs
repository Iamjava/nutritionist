use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::Router;
use axum::routing::{delete, get, post};
use tera::Tera;
use crate::app::forms::ProductForm;
use crate::app::handler;
use crate::models::models::NutritionistSearchQuery;

// static once cell for Templates with tera
pub static TEMPLATES: once_cell::sync::Lazy<Tera> = once_cell::sync::Lazy::new(|| {
    let tera = Tera::new("templates/**/*").unwrap();
    tera
});


pub async fn serve() {
    let port = std::env::var("NUT_PORT").unwrap_or("8000".to_string());
    let app = Router::new()
        .route("/", get(|| async { Response::builder().status(StatusCode::SEE_OTHER).header("Location", "/search").body("".to_string()).unwrap() }))
        .route("/search", post(|x: axum::Form<NutritionistSearchQuery>| async { handler::handle_search_post(x).await }))
        .route("/:id/search", post(handler::handle_search_post_meal_add))
        .route("/search", get(|| async { handler::handle_search().await }))
        .route("/test", post(|| async { handler::handle_search_test().await }))
        .route("/newmeal", get(|| async { handler::handle_create_meal().await }))
        .route("/meals", get(|| async { handler::handle_meals().await }))
        .route("/meals/:id/search", get(|id: Path<String>| async { handler::handle_search_meal_add(id).await }))
        .route("/meals/:id", get(|id: Path<String>| async { handler::handle_meal(id).await }))
        .route("/meals/:id", post(|id: Path<String>,x: axum::Form<ProductForm> | async { handler::handle_add_content_to_meal(id, x).await }))
        .route("/meals/:id/:code", delete(|info: Path<(String,String)>| async { handler::remove_product_from_meal_handler(info).await }))
    ;

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}