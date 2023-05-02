use once_cell::sync::Lazy;
use scraper::Selector;

const NUM_LIMIT: usize = 10000;

static SUBTITLE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".subtitle a").unwrap());
pub(crate) fn extract(html: &str) -> Vec<String> {
    let html = scraper::Html::parse_document(html);
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
    subtitles
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
    assert_eq!(
        subtitles,
        &[
            "First first first first",
            "Second second second second",
            "Third third third third"
        ]
    );
}
