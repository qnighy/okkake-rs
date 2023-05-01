mod atom;
mod ncode;

use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use time::OffsetDateTime;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn atom() -> impl IntoResponse {
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
            href: "https://example.com/".to_owned(),
        }],
        id: atom::Id {
            text: "https://example.com/".to_owned(),
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
                    href: "https://example.com/article/1".to_owned(),
                }],
                id: atom::Id {
                    text: "https://example.com/article/1".to_owned(),
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
                    href: "https://example.com/article/2".to_owned(),
                }],
                id: atom::Id {
                    text: "https://example.com/article/2".to_owned(),
                },
            },
        ],
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
