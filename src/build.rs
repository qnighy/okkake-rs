use time::OffsetDateTime;

use crate::atom;
use crate::ncode::Ncode;
use crate::scraping::NovelData;

pub(crate) fn build_feed(
    base: &str,
    id: Ncode,
    novel_data: &NovelData,
    now: OffsetDateTime,
) -> atom::Feed {
    atom::Feed {
        title: format!("【再】{}", novel_data.novel_title),
        subtitle: format!("『{}』の既存話を再配信します。", novel_data.novel_title),
        updated: now,
        generator: atom::Generator {
            version: "0.1.0".to_owned(),
            name: "Okkake-rs".to_owned(),
        },
        links: vec![atom::Link {
            rel: "self".to_owned(),
            type_: "application/atom+xml".to_owned(),
            href: format!("{}/novels/{}/atom.xml", base, id),
        }],
        id: format!("{}/novels/{}/atom.xml", base, id),
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
    }
}
