use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::atom;
use crate::ncode::Ncode;

const DAYS: u64 = 100;

#[derive(Debug, Clone)]
pub(crate) struct BuildFeedParams<'a> {
    pub(crate) base: &'a str,
    pub(crate) id: Ncode,
    pub(crate) author: &'a str,
    pub(crate) title: &'a str,
    pub(crate) start: OffsetDateTime,
    pub(crate) now: OffsetDateTime,
    pub(crate) category: crate::Category,
}

pub(crate) fn build_feed(
    BuildFeedParams {
        base,
        id,
        author,
        title,
        start,
        now,
        category,
    }: BuildFeedParams<'_>,
) -> atom::Feed {
    let url = url::Url::parse_with_params(
        &format!("{}/{}/{}/atom.xml", base, category.novels_name(), id),
        &[("start", &start.format(&Rfc3339).unwrap())],
    )
    .unwrap();

    let max_days = Ord::max((now - start).whole_days() + 1, 0) as u64;
    let min_days = max_days.saturating_sub(DAYS);
    atom::Feed {
        title: format!("【再】{}", title),
        subtitle: format!("『{}』の既存話を再配信します。", title),
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
            name: author.to_owned(),
            uri: None,
        },
        entries: (min_days..max_days)
            .rev()
            .map(|day| {
                // We do not cap episode; let user unsubscribe it when the feed exceeds the limit
                let ep = day;
                let ep_time = start + Duration::days(day as i64);
                let id_url = url::Url::parse_with_params(
                    &format!("{}/{}/{}/{}/", base, category.novels_name(), id, day + 1),
                    &[("start", &start.format(&Rfc3339).unwrap())],
                )
                .unwrap();
                let title = format!("連載小説[{}](第{}部分【再】)", title, ep + 1);
                atom::Entry {
                    title,
                    published: ep_time,
                    updated: ep_time,
                    links: vec![atom::Link {
                        rel: "alternate".to_owned(),
                        type_: "text/html".to_owned(),
                        href: format!(
                            "https://{}.syosetu.com/{}/{}/",
                            category.subdomain(),
                            id,
                            ep + 1
                        ),
                    }],
                    id: id_url.to_string(),
                }
            })
            .collect::<Vec<_>>(),
    }
}
