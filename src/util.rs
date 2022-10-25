use std::{fs, time::SystemTime};

use chrono::{DateTime, FixedOffset, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::{info, warn};
use regex::Regex;
use std::fmt::Write as _;

use crate::{
  blog::{Blog, Post},
  xml::parse_web_feed,
};

/// Downloads all the RSS feeds specified in `feeds.txt` and converts them to `Blog`s.
pub fn download_blogs(days: i64, feed_flag: usize, env_links: Vec<&str>) -> Vec<Blog> {
  let links = read_feeds(feed_flag, env_links);

  let contents: Vec<Blog> = links
    .into_iter()
    .filter(|link| !link.is_empty())
    .filter_map(|link| {
      let xml = get_page(&link)
        .map_err(|e| warn!("Error in {}\n{:?}", link, e))
        .ok()?;

      parse_web_feed(&xml)
        .map_err(|e| warn!("Error in {}\n{}", link, e))
        .ok()
    })
    .filter_map(|x| {
      if !within_n_days(days, &x.last_build_date) {
        return None;
      }

      let recent_posts: Vec<Post> = x
        .posts
        .into_iter()
        .filter(|x| within_n_days(days, &x.last_build_date))
        .collect();

      let non_empty = !recent_posts.is_empty();

      non_empty.then_some(Blog {
        posts: recent_posts,
        ..x
      })
    })
    .collect();

  contents
}

/// Parses links from `feeds.txt`.
///
/// Assumed one link per line. Any text between a `#` and a line end
/// is considered a comment.
fn read_feeds(feed_flag: usize, env_links: Vec<&str>) -> Vec<String> {    
  if feed_flag > 1 {
    // use env var for feeds
    let mut tmp: Vec<String> = Vec::new();
    for s in env_links {
      tmp.push(s.to_string());
    }        
    // testing
    /*println!("Env vars were used, now printing...");
    for s in &tmp {
      println!("{}",s);
    }*/
    return tmp;
  }

  let links = fs::read_to_string("feeds.txt").expect("Error in reading the feeds.txt file");

  // Not really necessary but yes
  // https://docs.rs/regex/latest/regex/#example-avoid-compiling-the-same-regex-in-a-loop
  lazy_static! {
    static ref RE: Regex = #[allow(clippy::unwrap_used)]
    Regex::new(r"#.*$").unwrap();
  }

  links
    .split('\n')
    .map(std::string::ToString::to_string)
    .map(|l| RE.replace_all(&l, "").to_string())
    .filter(|l| !l.is_empty())
    .map(|l| l.trim().to_owned())
    .unique()
    .collect::<Vec<String>>()
}

/// Generates the HTML contents corresponding to the given Blog collection.
pub fn map_to_html(blogs: &Vec<Blog>) -> String {
  let styling_part1 = format!("
    <!DOCTYPE html>
    <html lang=\"en\">
        <head>
            <meta charset=\"utf-8\" />
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\" />
            <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/css/bootstrap.min.css\" integrity=\"sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T\" crossorigin=\"anonymous\">
        </head>
        <body>
            <nav class=\"navbar navbar-expand-lg navbar-dark bg-dark text-light\">
              <a class=\"navbar-brand\">Rss2Email Feed</a>
            </nav>
            <div class=\"container\">
                <div class=\"text-left mt-5\">
  ");
  
  let mut res = format!("<h1>Rss2Email - {}</h1>", Utc::now().date());

  for blog in blogs {
    let mut tmp = format!("<h2>{}</h2><ul>", blog.title);
    for post in &blog.posts {
      let _ = write!(tmp, "<li><a href=\"{}\">{}</a></li>", post.link, post.title);
    }
    tmp.push_str("</ul>");
    res.push_str(&tmp);
  }

  let styling_part2: String = format!("
      </div>
      </div>
      
      <!-- Bootstrap core JS-->
      <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/js/bootstrap.bundle.min.js\"></script>
    </body>
    </html>
  ");

  let mut res2: String = format!("");
  res2.push_str(&styling_part1);
  res2.push_str(&res);
  res2.push_str(&styling_part2);

  res2
  //res 
}

/// Returns true if the passed date is within `n` days from the current date.
fn within_n_days(n: i64, date: &DateTime<FixedOffset>) -> bool {
  let today = Utc::now();

  let tz = date.timezone();
  let today = today.with_timezone(&tz);
  (today - *date).num_days() <= n
}

/// Helper function for downloading the contents of a web page.
fn get_page(url: &str) -> Result<String, ureq::Error> {
  let body: String = ureq::get(url)
    .set("Example-Header", "header value")
    .call()?
    .into_string()?;

  Ok(body)
}

/// Helper function that times and prints the elapsed execution time
/// of `F` if ran in debug mode.
pub fn time_func<F, O>(f: F, fname: &str) -> O
where
  F: Fn() -> O,
  O: Clone,
{
  let start = SystemTime::now();

  let res = f();

  let since_the_epoch = SystemTime::now()
    .duration_since(start)
    .expect("Time went backwards");

  if cfg!(debug_assertions) {
    info!(
      "Elapsed time for {} was {:?}ms",
      fname,
      since_the_epoch.as_millis()
    );
  }

  res
}
