use std::fmt;

use anyhow::{Context, Result, bail};
use comrak::adapters::CodefenceRendererAdapter;
use comrak::html::{escape, escape_href};
use comrak::nodes::Sourcepos;
use serde::Deserialize;

use super::FriendLinks;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Friend {
    name: String,
    url: String,
    description: Option<String>,
    avatar: Option<String>,
}

impl CodefenceRendererAdapter for FriendLinks {
    fn write(
        &self,
        output: &mut dyn fmt::Write,
        _language: &str,
        _metadata: &str,
        code: &str,
        _source_position: Option<Sourcepos>,
    ) -> fmt::Result {
        let friends = match parse(code) {
            Ok(friends) => friends,
            Err(error) => {
                *self.error.lock().unwrap() = Some(error);

                return Err(fmt::Error);
            }
        };

        render(output, &friends)
    }
}

fn parse(source: &str) -> Result<Vec<Friend>> {
    let mut friends: Vec<Friend> = yaml_serde::from_str(source).context("invalid friends block")?;
    if friends.is_empty() {
        bail!("friends block cannot be empty");
    }

    for (index, friend) in friends.iter_mut().enumerate() {
        friend.name = friend.name.trim().to_owned();
        friend.url = friend.url.trim().to_owned();
        friend.description = friend
            .description
            .take()
            .map(|description| description.trim().to_owned())
            .filter(|description| !description.is_empty());
        friend.avatar = friend
            .avatar
            .take()
            .map(|avatar| avatar.trim().to_owned())
            .filter(|avatar| !avatar.is_empty());

        if friend.name.is_empty() {
            bail!("friends entry {} has an empty `name`", index + 1);
        }
        if friend.url.is_empty() {
            bail!("friends entry {} has an empty `url`", index + 1);
        }
    }

    Ok(friends)
}

fn render(output: &mut dyn fmt::Write, friends: &[Friend]) -> fmt::Result {
    output.write_str("<ul class=\"friend-links\">\n")?;
    for friend in friends {
        output.write_str("<li>\n<a class=\"friend-link\" href=\"")?;
        escape_href(output, &friend.url, false)?;
        output.write_str("\">\n<span class=\"friend-link-avatar\" aria-hidden=\"true\">")?;
        if let Some(avatar) = friend.avatar.as_deref() {
            output.write_str("<img src=\"")?;
            escape_href(output, avatar, false)?;
            output.write_str("\" alt=\"\" loading=\"lazy\" decoding=\"async\">")?;
        } else {
            let initial = friend.name.chars().next().unwrap().to_string();
            escape(output, &initial)?;
        }
        output.write_str(
            "</span>\n<span class=\"friend-link-content\">\n<span class=\"friend-link-name\">",
        )?;
        escape(output, &friend.name)?;
        output.write_str("</span>\n")?;
        if let Some(description) = friend.description.as_deref() {
            output.write_str("<span class=\"friend-link-description\">")?;
            escape(output, description)?;
            output.write_str("</span>\n")?;
        }
        output.write_str("</span>\n</a>\n</li>\n")?;
    }

    output.write_str("</ul>\n")
}
