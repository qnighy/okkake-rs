use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
#[serde(rename = "feed")]
pub(crate) struct Feed {
    #[serde(rename = "@xmlns")]
    pub(crate) xmlns: &'static str,
    pub(crate) title: Title,
    pub(crate) subtitle: Subtitle,
    pub(crate) updated: Updated,
    pub(crate) generator: Generator,
    pub(crate) links: Vec<Link>,
    pub(crate) id: Id,
    pub(crate) author: Author,
    pub(crate) entries: Vec<Entry>,
}

impl Feed {
    pub(crate) const XMLNS: &str = "http://www.w3.org/2005/Atom";
}

#[derive(Serialize)]
#[serde(rename = "title")]
pub(crate) struct Title {
    #[serde(rename = "@type")]
    pub(crate) type_: &'static str,
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "subtitle")]
pub(crate) struct Subtitle {
    #[serde(rename = "@type")]
    pub(crate) type_: &'static str,
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "updated")]
pub(crate) struct Updated {
    #[serde(rename = "$text", with = "time::serde::rfc3339")]
    pub(crate) value: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename = "published")]
pub(crate) struct Published {
    #[serde(rename = "$text", with = "time::serde::rfc3339")]
    pub(crate) value: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename = "generator")]
pub(crate) struct Generator {
    #[serde(rename = "@version")]
    pub(crate) version: &'static str,
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "link")]
pub(crate) struct Link {
    #[serde(rename = "@rel")]
    pub(crate) rel: &'static str,
    #[serde(rename = "@type")]
    pub(crate) type_: &'static str,
    #[serde(rename = "@href")]
    pub(crate) href: String,
}

#[derive(Serialize)]
#[serde(rename = "id")]
pub(crate) struct Id {
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "author")]
pub(crate) struct Author {
    pub(crate) name: Name,
    pub(crate) uri: Uri,
}

#[derive(Serialize)]
#[serde(rename = "name")]
pub(crate) struct Name {
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "uri")]
pub(crate) struct Uri {
    #[serde(rename = "$text")]
    pub(crate) text: String,
}

#[derive(Serialize)]
#[serde(rename = "entry")]
pub(crate) struct Entry {
    pub(crate) title: Title,
    pub(crate) published: Published,
    pub(crate) updated: Updated,
    pub(crate) links: Vec<Link>,
    pub(crate) id: Id,
}
