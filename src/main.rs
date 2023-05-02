mod atom;
mod ncode;
mod scraping;

use std::sync::Arc;

use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use scraping::NovelData;
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool};
use time::OffsetDateTime;

use crate::ncode::Ncode;
use crate::scraping::scrape;

#[derive(Debug)]
struct State {
    base: String,
    reqwest_client: reqwest::Client,
    db: PgPool,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom(Extension(state): Extension<Arc<State>>, Path(id): Path<Ncode>) -> impl IntoResponse {
    let novel_data = get_or_scrape(&state.reqwest_client, &state.db, id).await;
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

async fn get_or_scrape(client: &reqwest::Client, db: &PgPool, ncode: Ncode) -> NovelData {
    let now = OffsetDateTime::now_utc();
    let result = scrape(client, ncode).await;
    {
        let data = result.as_ref().ok().cloned().unwrap_or(NovelData {
            novel_title: Default::default(),
            subtitles: Default::default(),
        });
        let error = result.as_ref().err().map(|e| e.to_string());
        sqlx::query(
            "
INSERT INTO novel_data
(ncode, data, error, last_fetched_at)
VALUES ($1, $2, $3, $4)
ON CONFLICT (ncode)
DO UPDATE SET data = $2, error = $3, last_fetched_at = $4;",
        )
        .bind(ncode.to_string())
        .bind(sqlx::types::Json(data))
        .bind(error)
        .bind(now)
        .execute(db)
        .await
        .unwrap();
    }
    result.unwrap()
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.DEV_DATABASE_URL}")] db: PgPool,
) -> shuttle_axum::ShuttleAxum {
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

    db.execute(include_str!("../schema.sql")).await.unwrap();

    let state = State {
        base: "https://okkake.shuttleapp.rs".to_owned(),
        reqwest_client,
        db,
    };
    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/novels/:id/atom.xml", get(atom))
        .layer(Extension(Arc::new(state)));

    Ok(router.into())
}
