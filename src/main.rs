use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom() -> impl IntoResponse {
    ([(CONTENT_TYPE, "application/atom+xml")], "<>")
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/novels/:id/atom.xml", get(atom));

    Ok(router.into())
}
