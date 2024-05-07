use std::cmp::PartialEq;
use crate::app::forms::ProductForm;
use crate::db;
use crate::models::meal::{Meal, MealType};
use crate::models::models::RedisORM;
use crate::models::user::User;
use crate::usda::search::{Food, NutrientValues};
use askama::Template;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::Form;
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use uuid::Uuid;
use crate::models::meal::MealType::Snack;

#[derive(Template)] // this will generate the code...
#[template(path = "meals/meals_view.html")] // using the template in this path, relative
                                            // to the `templates` dir in the crate root
struct MealsTemplate<'a> {
    // the name of the struct can be anything
    meal_id: &'a str,
    username: &'a str,
    meals: Vec<Meal>,
    date: chrono::NaiveDate,
}

#[derive(Template)]
#[template(path = "meals/meal_view.html")]
pub struct MealView {
    pub(crate) meal: Meal,
    macros: NutrientValues,
    edit: bool,
}

#[derive(Template)] // this will generate the code...
#[template(path = "test/search.html")] // using the template in this path, relative
                                       // to the `templates` dir in the crate root
struct SearchTemplate<'a> {
    // the name of the struct can be anything
    meal_id: &'a str,
}

pub async fn handle_meals(claims: Option<OidcClaims<EmptyAdditionalClaims>>) -> Response<String> {
    let mut con = crate::db::connector::get_connection()
        .expect("Could not connect to redis,maybe redis is not running");
    let meals = Meal::all(&mut con);
    let claims = claims.unwrap();
    let username = claims.preferred_username().unwrap();
    // add meal to hashmap of meals where date is the key
    let mut hash_map = std::collections::HashMap::new();
    meals.iter().for_each(|meal| {
        if hash_map.contains_key(&meal.date.date_naive()) {
            let mut m: &mut Vec<Meal>  = hash_map.get_mut(&meal.date.date_naive()).unwrap();
            m.push(meal.clone());
        }else{
            hash_map.insert(meal.date.date_naive(), vec![meal.clone()]);
        }
    });
    //iterate hashmap and add to vector
    for (key, value) in hash_map.iter() {
        println!("{}: {:?}", key, value);
    }

    let t = MealsTemplate {
        meal_id: "test",
        meals,
        username: claims.preferred_username().unwrap(),
        date: chrono::Utc::now().date_naive(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.render().unwrap())
        .unwrap()
}


// Redirect to a newly created meal /meal/:id
pub async fn handle_create_meal(meal_type: Path<String>, oidc_claims: Option<OidcClaims<EmptyAdditionalClaims>>) -> Response<String> {
    let mut con = crate::db::connector::get_connection()
        .expect("Could not connect to redis,maybe redis is not running");
    let meal_type = match meal_type.to_string().as_str() {
        "lunch" => MealType::Lunch,
        "dinner" => MealType::Dinner,
        "snack" => Snack,
        _ => MealType::Breakfast,
    };
    let today = chrono::Utc::now().date_naive();
    //hier userspezifisch
    let creds = oidc_claims.unwrap();
    let user_name = creds.preferred_username().unwrap().to_string();
    // hier die meals from user holen
    let meals = Meal::all(&mut con);
    for meal in meals.iter() {
        if meal.date.date_naive() == today && meal.meal_type == meal_type {
            return Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header("Location", format!("/meals/{}", meal.id))
                .body("".into())
                .unwrap();
        }
    }

    let mut meal = Meal::example();
    meal.meal_type = meal_type;

    let mut con = crate::db::connector::get_connection()
        .expect("Could not connect to redis,maybe redis is not running");
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
    let mut con = crate::db::connector::get_connection()
        .expect("Could not connect to redis,maybe redis is not running");
    let mut user = User::example();
    user.id = "TEST_ID".to_string();
    let mut meal = Meal::fetch_from_uuid(&mut con, &id).expect("DIDNT FIND MEAL");
    // Using the tera Context struct
    for content in meal.contents.iter_mut() {
        let mut c = content.clone();
        content.product = c.product.generate_nutrient_values() * content.quantity * 0.01;
    }
    let macros = meal.get_macros();
    let t = MealView {
        meal,
        macros,
        edit: true,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.render().unwrap())
        .unwrap()
}

pub async fn remove_product_from_meal_handler(
    Path((meal_id, id)): Path<(String, String)>,
) -> Response<String> {
    let mut con = db::connector::get_connection().unwrap();
    let mut meal = Meal::fetch_from_uuid(&mut con, &meal_id).unwrap();
    dbg!(meal.contents.len());
    meal.contents
        .retain(|x| x.id != id.to_string().parse().unwrap());
    dbg!(meal.contents.len());
    for content in meal.contents.iter_mut() {
        let mut c = content.clone();
        content.product = c.product.generate_nutrient_values() * content.quantity * 0.01;
    }
    let macros = meal.get_macros();

    let macros = meal.get_macros();
    meal.save(&mut con).unwrap();
    let t = MealView {
        meal: meal,
        macros: macros,
        edit: true,
    };
    Response::builder()
        .status(StatusCode::OK)
        .body(t.render().unwrap())
        .unwrap()
}

pub async fn handle_search_meal_add(Path(id): Path<String>) -> Response<String> {
    let t = SearchTemplate { meal_id: &id };
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(t.render().unwrap())
        .unwrap()
}

// add a content to a meal /meal/:id/addcontent
pub async fn handle_add_content_to_meal(
    Path((id)): Path<String>,
    x: Form<ProductForm>,
) -> Response<String> {
    let mut con = crate::db::connector::get_connection()
        .expect("Could not connect to redis,maybe redis is not running");

    let mut meal = Meal::fetch_from_uuid(&mut con, &id).expect("DIDNT FIND MEAL");
    let prod = Food::fetch_from_uuid(&mut con, &x.product_code).expect("DIDNT FIND PRODUCT");
    let prod = crate::models::meal::MealContent {
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
