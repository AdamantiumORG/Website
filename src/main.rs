use axum::{Router, response::Html, routing::get};
use std::env;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(home))
        .nest_service("/static", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Unable to bind to address");

    println!("Server is running at http://localhost:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Error while serving the server");
}

async fn home() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}
