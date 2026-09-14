use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use pulldown_cmark::{Options, Parser, html};
use std::{
    env,
    path::{Component, Path as FilePath},
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/", get(home))
        .route("/download", get(download))
        .route("/docs", get(docs))
        .route("/docs/", get(docs))
        .route("/docs/{*path}", get(doc_page))
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

async fn download() -> Html<&'static str> {
    Html(include_str!("../static/download.html"))
}

async fn docs() -> Redirect {
    Redirect::temporary("/docs/README.md")
}

async fn doc_page(Path(path): Path<String>) -> Result<Html<String>, StatusCode> {
    let relative = FilePath::new(&path);
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("md")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let root = tokio::fs::canonicalize(FilePath::new(env!("CARGO_MANIFEST_DIR")).join("docs"))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file = tokio::fs::canonicalize(root.join(relative))
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !file.starts_with(&root) {
        return Err(StatusCode::NOT_FOUND);
    }

    let markdown = tokio::fs::read_to_string(file)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut content = String::new();
    html::push_html(
        &mut content,
        Parser::new_ext(
            &markdown,
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
        ),
    );
    Ok(Html(
        include_str!("../static/docs.html").replace("{{content}}", &content),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn renders_documentation_and_nested_pages() {
        let index = doc_page(Path("README.md".into())).await.unwrap().0;
        assert!(index.contains("href=\"language/variables.md\""));
        assert!(!index.contains("{{content}}"));
        let page = doc_page(Path("language/variables.md".into()))
            .await
            .unwrap()
            .0;
        assert!(page.contains("<h1>Variables</h1>"));
        assert!(page.contains("<pre><code"));
    }

    #[tokio::test]
    async fn rejects_missing_files_and_paths_outside_docs() {
        for path in [
            "missing.md",
            "../Cargo.toml",
            "../README.md",
            "language/../../README.md",
            "C:\\README.md",
        ] {
            assert_eq!(
                doc_page(Path(path.into())).await.unwrap_err(),
                StatusCode::NOT_FOUND
            );
        }
    }
}
