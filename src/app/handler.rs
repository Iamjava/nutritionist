use std::fmt::format;
use axum::http::{Response, StatusCode};
use std::fs;
use axum::extract::Path;
use axum::Form;
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use tera::Context;
use uuid::Uuid;
use crate::app::forms::ProductForm;
use crate::app::server::TEMPLATES;
use crate::db;
use crate::models::meal::{Meal, MealContent};
use crate::models::user::User;
use crate::models::models::{NutritionistSearchQuery, RedisORM};
use crate::models::product::Product;
use crate::open_food_facts::sdk::cached_search;

pub async fn handle_meals(
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
) -> Response<String>{
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let meals = Meal::all(&mut con);
    let mut context = Context::new();
    context.insert("meals", &meals);
    context.insert("username", &claims.unwrap().preferred_username());
    let t = TEMPLATES.render("meals/meals_view.html", &context).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.into())
        .unwrap()
}

fn to_html_search_item(item: Product) -> String {
    format!("<h3>{}, {:?} kcal / 100g</h3>", item.product_name.unwrap(), item.nutriments.unwrap().energy_kcal_100g)
}

pub async fn handle_search_post(x: axum::Form<NutritionistSearchQuery>) -> Response<String> {
    let product = cached_search(&*x.query).await.unwrap();
    let prods: Vec<_> = product.iter().filter(|p| p.product_name.is_some()).map(|x| to_html_search_item(x.clone())).collect();
    let string_from_template = format!("<h1>Search Results for {}</h1>{}", x.query, prods.join(""));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

pub async fn handle_search_test() -> Response<String> {
    let string_from_template = "TEST RESPONSE";

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

// add a content to a meal /meal/:id/addcontent
pub async fn handle_add_content_to_meal(Path((id)): Path<String>,x: Form<ProductForm>) -> Response<String> {
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");

    let mut meal = Meal::fetch_from_uuid(&mut con, &id).expect("DIDNT FIND MEAL");
    let prod = Product::fetch_from_uuid(&mut con, &x.product_code).expect("DIDNT FIND PRODUCT");
    let prod = crate::models::meal::MealContent{
        product: prod,
        quantity: x.amount,
        id: Uuid::new_v4(),
    };
    meal.contents.append(&mut vec![prod]);
    meal.save(&mut con).expect("DIDNT SAVE");

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", format!("/meals/{}", id))
        .body("".into())
        .unwrap()
}


// Redirect to a newly created meal /meal/:id
pub async fn handle_create_meal() -> Response<String> {
    let meal = Meal::example();
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    meal.save(&mut con).expect("DIDNT SAVE");
    // send a redirect to the newly created meal
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", format!("/meals/{}", meal.id))
        .body("".into())
        .unwrap()
}

// Display a meal
pub async fn handle_meal(Path(id): Path<String>) -> Response<String> {
    let mut con = crate::db::connector::get_connection().expect("Could not connect to redis,maybe redis is not running");
    let mut user= User::example();
    user.id = "TEST_ID".to_string();
    let meal = Meal::fetch_from_uuid(&mut con, &id).expect("DIDNT FIND MEAL");
    // Using the tera Context struct
    let macros = meal.get_macros();
    let mut context = Context::new();
    context.insert("meal", &meal);
    context.insert("macros", &macros);
    context.insert("edit", &true);
    let t = TEMPLATES.render("meals/meal_view.html", &context).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.into())
        .unwrap()
}

pub async fn handle_search_meal_add(Path(id): Path<String>) -> Response<String> {

    let mut context = Context::new();
    context.insert("meal_id", &id);
    let t = TEMPLATES.render("test/search.html", &context).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.into())
        .unwrap()
}
pub async fn handle_search(
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
) -> Response<String> {
    let string_from_template = fs::read_to_string("templates/test/search.html").unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

pub async fn handle_search_post_meal_add( Path(id): Path<String>, x: axum::Form<NutritionistSearchQuery>,) -> Response<String> {
    let product = cached_search(&*x.query).await;
    if product.is_err() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("No product found".into())
            .unwrap();
    }
    let product = product.unwrap();
    let mut context = Context::new();
    context.insert("products", &product);
    context.insert("mealid", &id);
    let string_from_template = TEMPLATES.render("product/search_response.html", &context).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(string_from_template.into())
        .unwrap()
}

pub async fn remove_product_from_meal_handler(Path((meal_id,id)): Path<(String,String)>) -> Response<String> {
    let mut con = db::connector::get_connection().unwrap();
    let mut meal = Meal::fetch_from_uuid(&mut con, &meal_id).unwrap();
    dbg!(meal.contents.len());
    meal.contents.retain(|x| x.id != id.to_string().parse().unwrap());
    dbg!(meal.contents.len());

    let macros = meal.get_macros();
    let mut context = Context::new();
    context.insert("meal", &meal);
    context.insert("macros", &macros);
    context.insert("edit", &true);


    let t = TEMPLATES.render("meals/meal_view.html", &context).unwrap();
    meal.save(&mut con).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(t.into())
        .unwrap()
}

pub(crate) async fn show_product_handler(code: Path<String>) ->Response<String> {
    let mut con = db::connector::get_connection().unwrap();
    let product = Product::fetch_from_uuid(&mut con, &code).unwrap();
    let macros = product.get_numerical_macros();
    let mut context = Context::new();
    context.insert("product", &product);
    context.insert("macros", &macros);
    let t = TEMPLATES.render("product/product_view.html", &context).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(t.into())
        .unwrap()
}