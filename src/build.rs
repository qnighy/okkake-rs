use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::atom;
use crate::ncode::Ncode;
use crate::scraping::NovelData;

const DAYS: u64 = 100;

pub(crate) fn build_feed(
    base: &str,
    id: Ncode,
    novel_data: &NovelData,
    start: OffsetDateTime,
    now: OffsetDateTime,
) -> atom::Feed {
    let url = url::Url::parse_with_params(
        &format!("{}/novels/{}/atom.xml", base, id),
        &[("start", &start.format(&Rfc3339).unwrap())],
    )
    .unwrap();

    let max_days = Ord::max((now - start).whole_days() + 1, 0) as u64;
    let min_days = max_days.saturating_sub(DAYS);
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
            href: url.to_string(),
        }],
        id: url.to_string(),
        author: atom::Author {
            name: novel_data.author.clone(),
            uri: None,
        },
        entries: (min_days..max_days)
            .rev()
            .map(|day| {
                let ep = Ord::min(day as usize, novel_data.subtitles.len() - 1);
                let ep_time = start + Duration::days(day as i64);
                let id_url = url::Url::parse_with_params(
                    &format!("{}/novels/{}/{}/", base, id, day + 1),
                    &[("start", &start.format(&Rfc3339).unwrap())],
                )
                .unwrap();
                atom::Entry {
                    title: novel_data.subtitles[ep].clone(),
                    published: ep_time,
                    updated: ep_time,
                    links: vec![atom::Link {
                        rel: "alternate".to_owned(),
                        type_: "text/html".to_owned(),
                        href: format!("https://ncode.syosetu.com/{}/{}/", id, ep + 1),
                    }],
                    id: id_url.to_string(),
                }
            })
            .collect::<Vec<_>>(),
    }
}
