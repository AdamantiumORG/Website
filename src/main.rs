use axum::{response::Html, routing::get, Router};
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new().route("/", get(home));

    // 4. Uruchom serwer
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Unable to bind to address");

    println!("Server is running at http://localhost:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Error while serving the server");
}

// Funkcja obsługująca stronę główną
async fn home() -> Html<&'static str> {
    Html("<h1>Hello World!</h1>")
}