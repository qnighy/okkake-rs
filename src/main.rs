use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Serialize;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom() -> impl IntoResponse {
    let feed = Feed {
        xmlns: Feed::XMLNS,
        title: Title {
            type_: "text",
            text: "タイトル".to_owned(),
        },
    };
    let feed = quick_xml::se::to_string(&feed).unwrap();
    ([(CONTENT_TYPE, "application/atom+xml")], feed)
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/novels/:id/atom.xml", get(atom));

    Ok(router.into())
}

#[derive(Serialize)]
#[serde(rename = "feed")]
struct Feed {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    title: Title,
}

#[derive(Serialize)]
#[serde(rename = "title")]
struct Title {
    #[serde(rename = "@type")]
    type_: &'static str,
    #[serde(rename = "$text")]
    text: String,
}

impl Feed {
    const XMLNS: &str = "http://www.w3.org/2005/Atom";
}
