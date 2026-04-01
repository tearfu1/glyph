mod auth;
mod config;
mod errors;
mod handlers;
mod models;
mod services;
mod validation;

use std::sync::Arc;

use axum::{extract::FromRef, routing::{delete, get, post, put}, Extension, Router};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Arc<AppConfig>,
}

impl FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> sqlx::PgPool {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Arc<AppConfig> {
        state.config.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "glyph_backend=debug,tower_http=debug,axum=trace".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env().expect("Failed to load configuration");
    let config = Arc::new(config);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations applied successfully");

    let bind_addr = format!("{}:{}", config.server_host, config.server_port);

    let state = AppState { pool, config };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_routes = Router::new()
        .route("/login", post(auth::handlers::login))
        .route("/register", post(auth::handlers::register))
        .route("/refresh", post(auth::handlers::refresh));

    let book_routes = Router::new()
        .route("/", get(handlers::books::get_books).post(handlers::books::add_book))
        .route("/{id}", get(handlers::books::get_book))
        .route("/shelf/{userId}", get(handlers::books::get_shelf))
        .route("/author/{authorId}", get(handlers::books::get_by_author))
        .route("/{bookId}/reviews", get(handlers::reviews::get_reviews).post(handlers::reviews::create_review))
        .route("/{bookId}/reviews/my", get(handlers::reviews::get_my_review))
        .route("/{bookId}/questions", get(handlers::questions::get_questions).post(handlers::questions::create_question))
        .route("/{bookId}/questions/my", get(handlers::questions::get_my_questions_for_book))
        .route("/{bookId}/questions/best", get(handlers::questions::get_best_questions))
        .route("/{bookId}/reading-status", post(handlers::reading_status::set_status));

    let review_item_routes = Router::new()
        .route("/{id}", put(handlers::reviews::update_review).delete(handlers::reviews::delete_review))
        .route("/{id}/reaction", post(handlers::reviews::add_reaction).delete(handlers::reviews::remove_reaction));

    let question_item_routes = Router::new()
        .route("/{id}", put(handlers::questions::update_question).delete(handlers::questions::delete_question))
        .route("/{id}/reaction", post(handlers::questions::add_reaction).delete(handlers::questions::remove_reaction))
        .route("/{id}/answer", post(handlers::answers::create_answer))
        .route("/incoming", get(handlers::questions::get_incoming_questions))
        .route("/answered", get(handlers::questions::get_answered_questions))
        .route("/my", get(handlers::questions::get_my_questions));

    let user_routes = Router::new()
        .route("/{id}/profile", get(handlers::users::get_user_profile))
        .route("/{id}/reviews", get(handlers::users::get_user_reviews))
        .route("/me/settings", get(handlers::users::get_user_settings))
        .route("/me/navigation", get(handlers::users::get_user_navigation))
        .route("/me", put(handlers::users::update_user));

    let admin_routes = Router::new()
        .route("/groups", get(handlers::users::get_all_groups))
        .route("/users/{id}/groups", put(handlers::users::update_user_groups));

    let app = Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/books", book_routes)
        .nest("/api/reviews", review_item_routes)
        .nest("/api/questions", question_item_routes)
        .route("/api/tags", get(handlers::tags::get_tags))
        .route("/api/reading-statuses", get(handlers::reading_status::get_statuses))
        .nest("/api/users", user_routes)
        .nest("/api/admin", admin_routes)
        .layer(Extension(state.config.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("Server error");
}
