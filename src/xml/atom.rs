//! [Specification](https://www.rfc-editor.org/rfc/rfc4287)
//!
//! ```xml
//! <feed>
//!   <title></title>
//!   <updated>ISO.8601</updated>
//!   <entry>
//!     <title></title>
//!     <link href=""/>
//!     <updated>ISO.8601</updated>
//!     <summary></summary>?
//!   </entry>
//! </feed>
//! ```

use chrono::{DateTime, Utc};
use quick_xml::DeError;
use serde_derive::{Deserialize, Serialize};

use crate::blog::{Blog, Post};

use super::{
  traits::{BlogPost, WebFeed},
  ParserError,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "feed")]
pub struct AtomFeed {
  pub title: String,
  #[serde(rename = "entry")]
  pub entries: Option<Vec<AtomPost>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "entry")]
pub struct AtomPost {
  pub title: String,
  #[serde(rename = "link")]
  pub links: Vec<Link>,
  pub summary: Option<String>,
  pub updated: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Link {
  href: String,
}

impl WebFeed for Result<AtomFeed, DeError> {
  fn into_blog(self) -> Result<Blog, ParserError> {
    let feed = self?;

    let title = feed.title;
    let posts: Vec<Post> = feed.entries.map_or_else(std::vec::Vec::new, |entries| {
      entries
        .iter()
        .filter_map(|x| x.clone().into_post().ok())
        .collect()
    });
    if posts.is_empty() {
      return Err(ParserError::Parse(format!("Empty feed: {}", title)));
    }

    let last_build_date = posts
      .iter()
      .map(|x| x.last_build_date)
      .max()
      .ok_or_else(|| ParserError::Parse("Date error.".to_owned()))?;

    Ok(Blog {
      title,
      last_build_date,
      posts,
    })
  }
}

impl WebFeed for AtomFeed {
  fn into_blog(self) -> Result<Blog, ParserError> {
    let title = self.title;
    let posts: Vec<Post> = self.entries.map_or_else(std::vec::Vec::new, |entries| {
      entries
        .iter()
        .filter_map(|x| x.clone().into_post().ok())
        .collect()
    });
    if posts.is_empty() {
      return Err(ParserError::Parse(format!("Empty feed: {}", title)));
    }

    let last_build_date = posts
      .iter()
      .map(|x| x.last_build_date)
      .max()
      .ok_or_else(|| ParserError::Parse("Date error.".to_owned()))?;

    Ok(Blog {
      title,
      last_build_date,
      posts,
    })
  }
}

impl BlogPost for AtomPost {
  fn into_post(self) -> Result<Post, ParserError> {
    let title = self.title;
    // Use the first link for now
    let link = self.links[0].href.clone();
    let description = self.summary;
    let pub_date = self.updated;

    match DateTime::parse_from_rfc3339(&pub_date) {
      Ok(last_build_date) => Ok(Post {
        title,
        link,
        description,
        last_build_date: last_build_date.with_timezone(&Utc),
      }),
      Err(e) => Err(ParserError::Date(e.to_string())),
    }
  }
}
