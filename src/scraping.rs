use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ncode::Ncode;

#[derive(Debug, Error)]
pub(crate) enum ScrapingError {
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("extract error: {0}")]
    ExtractError(#[from] ExtractError),
}

pub(crate) async fn scrape(client: &Client, ncode: Ncode) -> Result<NovelData, ScrapingError> {
    let body = client
        .get(format!("https://ncode.syosetu.com/{}", ncode))
        .send()
        .await?
        .text()
        .await?;

    Ok(extract(&body)?)
}

const NUM_LIMIT: usize = 10000;

static NOVEL_TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".novel_title").unwrap());
static NOVEL_DESCRIPTION_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#novel_ex").unwrap());
static AUTHOR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".novel_writername").unwrap());
static SUBTITLE_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".subtitle a").unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NovelData {
    pub(crate) novel_title: String,
    pub(crate) novel_description: String,
    pub(crate) author: String,
    pub(crate) subtitles: Vec<String>,
}

#[derive(Debug, Error)]
pub(crate) enum ExtractError {
    #[error("missing title")]
    MissingTitle,
    #[error("too many titles")]
    TooManyTitles,
    #[error("missing description")]
    MissingDesc,
    #[error("too many description")]
    TooManyDescs,
    #[error("missing author")]
    MissingAuthor,
    #[error("too many authors")]
    TooManyAuthor,
    #[error("no episode found")]
    NoEpisode,
}

pub(crate) fn extract(html: &str) -> Result<NovelData, ExtractError> {
    let html = scraper::Html::parse_document(html);
    let novel_title = textof(
        &html,
        &NOVEL_TITLE_SELECTOR,
        ExtractError::TooManyTitles,
        ExtractError::MissingTitle,
    )?;
    let novel_description = textof(
        &html,
        &NOVEL_DESCRIPTION_SELECTOR,
        ExtractError::TooManyDescs,
        ExtractError::MissingDesc,
    )?;
    let author = textof(
        &html,
        &AUTHOR_SELECTOR,
        ExtractError::TooManyAuthor,
        ExtractError::MissingAuthor,
    )?;
    let author = author.trim();
    let author = author.strip_prefix("作者：").unwrap_or(author).to_owned();

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
    if subtitles.is_empty() {
        return Err(ExtractError::NoEpisode);
    }
    Ok(NovelData {
        novel_title,
        novel_description,
        author,
        subtitles,
    })
}

fn textof(
    html: &Html,
    sel: &Selector,
    e1: ExtractError,
    e2: ExtractError,
) -> Result<String, ExtractError> {
    let elems = html.select(sel).take(2).collect::<Vec<_>>();
    let [elem] = elems[..] else {
            if elems.len() > 1 {
                return Err(e1);
            } else {
                return Err(e2);
            }
        };
    Ok(elem.text().collect::<Vec<_>>().concat())
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
    let subtitles = extract(html).unwrap();
    assert_eq!(subtitles.novel_title, "Novel title novel title novel title");
    assert_eq!(
        subtitles.novel_description,
        "\u{3000}Description description description description description description description description description description description description description description description"
    );
    assert_eq!(subtitles.author, "Author author author");
    assert_eq!(
        subtitles.subtitles,
        &[
            "First first first first",
            "Second second second second",
            "Third third third third"
        ]
    );
}
