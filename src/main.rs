mod atom;
mod build;
mod ncode;

use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use build::BuildFeedParams;
use serde::Deserialize;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::build::build_feed;
use crate::ncode::Ncode;

#[derive(Debug)]
struct State {
    base: String,
}

async fn root() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/html; charset=UTF-8")],
        include_str!("../public/index.html"),
    )
}

#[derive(Debug, Error)]
enum AtomError {
    #[error("Redirecting to: {0}")]
    Redirect(String),
}

impl IntoResponse for AtomError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AtomError::Redirect(url) => (
                axum::http::StatusCode::TEMPORARY_REDIRECT,
                [("Location", &url[..])],
            )
                .into_response(),
            #[allow(unreachable_patterns)]
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
    author: Option<String>,
    title: Option<String>,
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

    let now = OffsetDateTime::now_utc();
    let feed = build_feed(BuildFeedParams {
        base: &state.base,
        id,
        author: params.author.as_deref().unwrap_or(""),
        title: params.title.as_deref().unwrap_or(""),
        start,
        now,
    });
    let feed = feed.to_xml();
    Ok((
        [(CONTENT_TYPE, "application/atom+xml; charset=UTF-8")],
        feed,
    ))
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let state = State {
        base: "https://okkake.shuttleapp.rs".to_owned(),
    };
    let router = Router::new()
        .route("/", get(root))
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
