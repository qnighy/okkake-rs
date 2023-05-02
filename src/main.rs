mod atom;
mod ncode;
mod scraping;

use std::sync::Arc;

use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use shuttle_secrets::SecretStore;
use time::OffsetDateTime;

use crate::ncode::Ncode;
use crate::scraping::scrape;

#[derive(Debug)]
struct State {
    base: String,
    reqwest_client: reqwest::Client,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom(Extension(state): Extension<Arc<State>>, Path(id): Path<Ncode>) -> impl IntoResponse {
    let novel_data = scrape(&state.reqwest_client, id).await.unwrap();
    eprintln!("novel_data = {:#?}", novel_data);
    let now = OffsetDateTime::now_utc();
    let feed = atom::Feed {
        title: "Title title title".to_owned(),
        subtitle: "Subtitle subtitle subtitle".to_owned(),
        updated: now,
        generator: atom::Generator {
            version: "0.1.0".to_owned(),
            name: "Okkake-rs".to_owned(),
        },
        links: vec![atom::Link {
            rel: "self".to_owned(),
            type_: "application/atom+xml".to_owned(),
            href: format!("{}/novels/{}/atom.xml", state.base, id),
        }],
        id: format!("{}/novels/{}/atom.xml", state.base, id),
        author: atom::Author {
            name: "Author author author".to_owned(),
            uri: "https://example.com/author/author/author".to_owned(),
        },
        entries: vec![
            atom::Entry {
                title: "Title title title".to_owned(),
                published: now,
                updated: now,
                links: vec![atom::Link {
                    rel: "alternate".to_owned(),
                    type_: "text/html".to_owned(),
                    href: format!("https://ncode.syosetu.com/{}/1/", id),
                }],
                id: format!("https://ncode.syosetu.com/{}/1/", id),
            },
            atom::Entry {
                title: "Title title title".to_owned(),
                published: now,
                updated: now,
                links: vec![atom::Link {
                    rel: "alternate".to_owned(),
                    type_: "text/html".to_owned(),
                    href: format!("https://ncode.syosetu.com/{}/2/", id),
                }],
                id: format!("https://ncode.syosetu.com/{}/2/", id),
            },
        ],
    };
    let feed = feed.to_xml();
    ([(CONTENT_TYPE, "application/atom+xml")], feed)
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let bot_author_email = secret_store.get("BOT_AUTHOR_EMAIL");
    let reqwest_client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("User-Agent", "okkake-rs/0.1.0".parse().unwrap());
            if let Some(bot_author_email) = &bot_author_email {
                headers.insert("From", bot_author_email.parse().unwrap());
            }
            headers
        })
        .build()
        .unwrap();
    eprintln!("client = {:?}", reqwest_client);
    let state = State {
        base: "https://okkake.shuttleapp.rs".to_owned(),
        reqwest_client,
    };
    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/novels/:id/atom.xml", get(atom))
        .layer(Extension(Arc::new(state)));

    Ok(router.into())
}
