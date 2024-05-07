use crate::app::forms::ProductForm;
use crate::app::meal_handler::{handle_create_meal, handle_meals};
use crate::app::{handler, meal_handler};
use axum::error_handling::HandleErrorLayer;
use axum::extract::Path;
use axum::http::{Response, StatusCode, Uri};
use axum::routing::{delete, get, post};
use axum::{response::IntoResponse, Router};
use axum_oidc::{
    error::MiddlewareError, EmptyAdditionalClaims, OidcAuthLayer, OidcClaims, OidcLoginLayer,
    OidcRpInitiatedLogout,
};
use tower::ServiceBuilder;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

// static once cell for Templates with tera

pub async fn serve() {
    dotenvy::dotenv().ok();
    let app_url = std::env::var("APP_URL").expect("APP_URL env variable");
    let issuer = std::env::var("ISSUER").expect("ISSUER env variable");
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID env variable");
    let client_secret = std::env::var("CLIENT_SECRET").ok();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(120)));

    let oidc_login_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            e.into_response()
        }))
        .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

    let oidc_auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            e.into_response()
        }))
        .layer(
            OidcAuthLayer::<EmptyAdditionalClaims>::discover_client(
                Uri::from_maybe_shared(app_url).expect("valid APP_URL"),
                issuer,
                client_id,
                client_secret,
                vec!["email".to_string()],
            )
            .await
            .unwrap(),
        );

    let port = std::env::var("NUT_PORT").unwrap_or("8000".to_string());
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header("Location", "/meals")
                    .body("".to_string())
                    .unwrap()
            }),
        )
        .route("/:id/search", post(handler::search_usda_handler))
        .route(
            "/newmeal/:type/:date",
            get(handle_create_meal),
        )
        .route("/meals", get(handle_meals))
        .route(
            "/meals/:id/search",
            get(|id: Path<String>| async { meal_handler::handle_search_meal_add(id).await }),
        )
        .route(
            "/meals/:id",
            get(|id: Path<String>| async { meal_handler::handle_meal(id).await }),
        )
        .route(
            "/meals/:id",
            post(|id: Path<String>, x: axum::Form<ProductForm>| async {
                meal_handler::handle_add_content_to_meal(id, x).await
            }),
        )
        .route(
            "/meals/:id/:code",
            delete(|info: Path<(String, String)>| async {
                meal_handler::remove_product_from_meal_handler(info).await
            }),
        )
        .route("/foo/:id", get(authenticated))
        .route("/usda", get(handler::search_usda_handler))
        .route("/logout", get(logout))
        .layer(oidc_login_service)
        //.route("/", get(maybe_authenticated))
        .layer(oidc_auth_service)
        .layer(session_layer);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn authenticated(
    claims: OidcClaims<EmptyAdditionalClaims>,
    id: Path<String>,
) -> impl IntoResponse {
    format!(
        "Hello {} {:?} {:?}",
        claims.subject().as_str(),
        claims.email().expect("mail broken"),
        claims.preferred_username().expect("username broken")
    )
}

async fn maybe_authenticated(
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
) -> impl IntoResponse {
    if let Some(claims) = claims {
        format!(
            "Hello {:?}! You are already logged in from another Handler.",
            claims.subject()
        )
    } else {
        "Hello anon!".to_string()
    }
}

async fn logout(logout: OidcRpInitiatedLogout) -> impl IntoResponse {
    logout.with_post_logout_redirect(Uri::from_static("https://google.de"))
}
