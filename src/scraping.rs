use once_cell::sync::Lazy;
use scraper::Selector;

const NUM_LIMIT: usize = 10000;

static NOVEL_TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".novel_title").unwrap());
static SUBTITLE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".subtitle a").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NovelData {
    pub(crate) novel_title: String,
    pub(crate) subtitles: Vec<String>,
}
pub(crate) fn extract(html: &str) -> NovelData {
    let html = scraper::Html::parse_document(html);
    let mut novel_title = String::from("");
    for elem in html.select(&NOVEL_TITLE_SELECTOR) {
        let s = elem.text().collect::<Vec<_>>().concat();
        if !s.is_empty() {
            novel_title = s;
            break;
        }
    }
    let mut subtitles = Vec::new();
    for elem in html.select(&SUBTITLE_SELECTOR) {
        let Some(href) = elem.value().attr("href") else {
            continue;
        };
        let Some(num) = pathnum(href) else {
            continue;
        };
        if num > NUM_LIMIT {
            continue;
        }
        while subtitles.len() <= num {
            subtitles.push(String::from(""))
        }
        subtitles[num] = elem.text().collect::<Vec<_>>().concat();
    }
    NovelData {
        novel_title,
        subtitles,
    }
}

fn pathnum(href: &str) -> Option<usize> {
    // ["", "n4830bu", "1", ""]
    let mut parts = href.split("/");
    if parts.next() != Some("") {
        return None;
    }
    if parts.next().is_none() {
        return None;
    }
    let Some(num_part) = parts.next() else {
        return None;
    };
    let Ok(num_part) = num_part.parse::<usize>() else {
        return None;
    };
    if num_part == 0 {
        return None;
    }
    Some(num_part - 1)
}

#[test]
fn test_extract() {
    let html = include_str!("../tests/sample.html");
    let subtitles = extract(html);
    assert_eq!(subtitles.novel_title, "Novel title novel title novel title");
    assert_eq!(
        subtitles.subtitles,
        &[
            "First first first first",
            "Second second second second",
            "Third third third third"
        ]
    );
}
