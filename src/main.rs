mod atom;
mod build;
mod ncode;
mod scraping;

use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use rand::Rng;
use serde::Deserialize;
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::build::build_feed;
use crate::ncode::Ncode;
use crate::scraping::{scrape, NovelData, ScrapingError};

#[derive(Debug)]
struct State {
    base: String,
    reqwest_client: reqwest::Client,
    db: PgPool,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, Error)]
enum AtomError {
    #[error("Redirecting to: {0}")]
    Redirect(String),
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error),
    #[error("DB error: {0}")]
    ScrapingError(#[from] ScrapingError),
}

impl IntoResponse for AtomError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AtomError::Redirect(url) => (
                axum::http::StatusCode::TEMPORARY_REDIRECT,
                [("Location", &url[..])],
            )
                .into_response(),
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}", self),
            )
                .into_response(),
        }
    }
}

#[derive(Deserialize)]
struct AtomParams {
    #[serde(default, deserialize_with = "parse_rfc3339")]
    start: Option<OffsetDateTime>,
}

async fn atom(
    Extension(state): Extension<Arc<State>>,
    Path(id): Path<Ncode>,
    Query(params): Query<AtomParams>,
) -> Result<impl IntoResponse, AtomError> {
    let Some(start) = params.start else {
        let start = OffsetDateTime::now_utc()
            .replace_nanosecond(0).unwrap()
            .replace_second(0).unwrap();
        let url = url::Url::parse_with_params(
            &format!("http://example.com/novels/{}/atom.xml", id),
            &[("start", &start.format(&Rfc3339).unwrap())]
        ).unwrap();
        return Err(AtomError::Redirect(format!("{}?{}", url.path(), url.query().unwrap())));
    };

    let novel_data = get_or_scrape(&state.reqwest_client, &state.db, id).await?;
    let now = OffsetDateTime::now_utc();
    let feed = build_feed(&state.base, id, &novel_data, start, now);
    let feed = feed.to_xml();
    Ok((
        [(CONTENT_TYPE, "application/atom+xml; charset=UTF-8")],
        feed,
    ))
}

async fn get_or_scrape(
    client: &reqwest::Client,
    db: &PgPool,
    ncode: Ncode,
) -> Result<NovelData, AtomError> {
    let now = OffsetDateTime::now_utc();
    let force_refresh_time = now - time::Duration::hours(24);
    let random_refresh_time = now - time::Duration::hours(12);
    let force_refresh_time_on_err = now - time::Duration::hours(2);
    let random_refresh_time_on_err = now - time::Duration::hours(1);

    #[derive(sqlx::FromRow)]
    struct Row {
        data: sqlx::types::Json<NovelData>,
        error: Option<String>,
        last_fetched_at: PrimitiveDateTime,
    }

    let row = sqlx::query_as::<_, Row>(
        "
SELECT data, error, last_fetched_at
FROM novel_data
WHERE ncode = $1;",
    )
    .bind(ncode.to_string())
    .fetch_optional(db)
    .await?;

    if let Some(row) = &row {
        let last_fetched_at = row.last_fetched_at.assume_utc();
        let do_refresh = if row.error.is_none() {
            let t = force_refresh_time
                + (random_refresh_time - force_refresh_time)
                    * rand::thread_rng().gen_range(0.0..1.0);
            last_fetched_at < t
        } else {
            let t = force_refresh_time_on_err
                + (random_refresh_time_on_err - force_refresh_time_on_err)
                    * rand::thread_rng().gen_range(0.0..1.0);
            last_fetched_at < t
        };
        if !do_refresh {
            eprintln!("Reusing response");
            return Ok(if let Some(e) = &row.error {
                todo!("{}", e);
            } else {
                row.data.0.clone()
            });
        }
    }

    let result = scrape(client, ncode).await;
    if result.is_err() {
        if let Some(row) = &row {
            let last_fetched_at = row.last_fetched_at.assume_utc();
            if row.error.is_none() && last_fetched_at >= force_refresh_time {
                eprintln!("Reusing response");
                // Reuse cache on error
                return Ok(row.data.0.clone());
            }
        }
    }
    {
        let data = result.as_ref().ok().cloned().unwrap_or(NovelData {
            novel_title: Default::default(),
            novel_description: Default::default(),
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
        .await?;
    }
    eprintln!("Using fresh response");
    Ok(result?)
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

fn parse_rfc3339<'de, D>(de: D) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => OffsetDateTime::parse(s, &Rfc3339)
            .map_err(serde::de::Error::custom)
            .map(Some),
    }
}
