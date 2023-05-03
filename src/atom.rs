use std::io::Write;

use quick_xml::{events::BytesText, Writer};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub(crate) struct Feed {
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) updated: OffsetDateTime,
    pub(crate) generator: Generator,
    pub(crate) links: Vec<Link>,
    pub(crate) id: String,
    pub(crate) author: Author,
    pub(crate) entries: Vec<Entry>,
}

impl Feed {
    pub(crate) fn to_xml(&self) -> String {
        let mut s = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
            .as_bytes()
            .to_owned();
        self.write_to(&mut Writer::new(&mut s)).unwrap();
        String::from_utf8(s).unwrap()
    }
    fn write_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("feed")
            .with_attribute(("xmlns", "http://www.w3.org/2005/Atom"))
            .write_inner_content(|writer| self.write_inner_to(writer))?;
        Ok(())
    }
    fn write_inner_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("title")
            .with_attribute(("type", "text"))
            .write_text_content(BytesText::new(&self.title))?;

        writer
            .create_element("subtitle")
            .with_attribute(("type", "text"))
            .write_text_content(BytesText::new(&self.subtitle))?;

        let updated = self.updated.to_string();
        writer
            .create_element("updated")
            .write_text_content(BytesText::new(&updated))?;

        self.generator.write_to(writer)?;

        for link in &self.links {
            link.write_to(writer)?;
        }

        writer
            .create_element("id")
            .write_text_content(BytesText::new(&self.id))?;

        self.author.write_to(writer)?;

        for entry in &self.entries {
            entry.write_to(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Generator {
    pub(crate) version: String,
    pub(crate) name: String,
}

impl Generator {
    fn write_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("generator")
            .with_attribute(("version", &self.version[..]))
            .write_text_content(BytesText::new(&self.name))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Link {
    pub(crate) rel: String,
    pub(crate) type_: String,
    pub(crate) href: String,
}

impl Link {
    fn write_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("link")
            .with_attribute(("rel", &self.rel[..]))
            .with_attribute(("type", &self.type_[..]))
            .with_attribute(("href", &self.href[..]))
            .write_empty()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Author {
    pub(crate) name: String,
    pub(crate) uri: Option<String>,
}

impl Author {
    fn write_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("author")
            .write_inner_content(|writer| self.write_inner_to(writer))?;
        Ok(())
    }

    fn write_inner_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("name")
            .write_text_content(BytesText::new(&self.name))?;
        if let Some(uri) = &self.uri {
            writer
                .create_element("uri")
                .write_text_content(BytesText::new(uri))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) title: String,
    pub(crate) published: OffsetDateTime,
    pub(crate) updated: OffsetDateTime,
    pub(crate) links: Vec<Link>,
    pub(crate) id: String,
}

impl Entry {
    fn write_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("entry")
            .write_inner_content(|writer| self.write_inner_to(writer))?;
        Ok(())
    }

    fn write_inner_to<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("title")
            .with_attribute(("type", "text"))
            .write_text_content(BytesText::new(&self.title))?;

        let published = self.published.to_string();
        writer
            .create_element("published")
            .write_text_content(BytesText::new(&published))?;

        let updated = self.updated.to_string();
        writer
            .create_element("updated")
            .write_text_content(BytesText::new(&updated))?;

        for link in &self.links {
            link.write_to(writer)?;
        }

        writer
            .create_element("id")
            .write_text_content(BytesText::new(&self.id))?;

        Ok(())
    }
}
