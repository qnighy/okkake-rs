mod atom;
mod ncode;

use std::sync::Arc;

use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use time::OffsetDateTime;

use crate::ncode::Ncode;

#[derive(Debug)]
struct State {
    base: String,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom(Extension(state): Extension<Arc<State>>, Path(id): Path<Ncode>) -> impl IntoResponse {
    let now = OffsetDateTime::now_utc();
    let feed = atom::Feed {
        xmlns: atom::Feed::XMLNS,
        title: atom::Title {
            type_: "text",
            text: "Title title title".to_owned(),
        },
        subtitle: atom::Subtitle {
            type_: "text",
            text: "Subtitle subtitle subtitle".to_owned(),
        },
        updated: atom::Updated { value: now },
        generator: atom::Generator {
            version: "0.1.0",
            text: "Okkake-rs".to_owned(),
        },
        links: vec![atom::Link {
            rel: "self",
            type_: "application/atom+xml",
            href: format!("{}/novels/{}/atom.xml", state.base, id),
        }],
        id: atom::Id {
            text: format!("{}/novels/{}/atom.xml", state.base, id),
        },
        author: atom::Author {
            name: atom::Name {
                text: "Author author author".to_owned(),
            },
            uri: atom::Uri {
                text: "https://example.com/author/author/author".to_owned(),
            },
        },
        entries: vec![
            atom::Entry {
                title: atom::Title {
                    type_: "html",
                    text: "Title title title".to_owned(),
                },
                published: atom::Published { value: now },
                updated: atom::Updated { value: now },
                links: vec![atom::Link {
                    rel: "alternate",
                    type_: "text/html",
                    href: format!("https://ncode.syosetu.com/{}/1/", id),
                }],
                id: atom::Id {
                    text: format!("https://ncode.syosetu.com/{}/1/", id),
                },
            },
            atom::Entry {
                title: atom::Title {
                    type_: "html",
                    text: "Title title title".to_owned(),
                },
                published: atom::Published { value: now },
                updated: atom::Updated { value: now },
                links: vec![atom::Link {
                    rel: "alternate",
                    type_: "text/html",
                    href: format!("https://ncode.syosetu.com/{}/2/", id),
                }],
                id: atom::Id {
                    text: format!("https://ncode.syosetu.com/{}/2/", id),
                },
            },
        ],
    };
    let feed = quick_xml::se::to_string(&feed).unwrap();
    ([(CONTENT_TYPE, "application/atom+xml")], feed)
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let state = State {
        base: "https://okkake.qnighy.info".to_owned(),
    };
    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/novels/:id/atom.xml", get(atom))
        .layer(Extension(Arc::new(state)));

    Ok(router.into())
}
