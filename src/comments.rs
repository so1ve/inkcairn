use std::collections::{HashMap, HashSet};
use std::env;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::metadata::Giscus;

#[derive(Default)]
pub struct CommentSnapshot {
    discussions: Option<HashMap<String, Discussion>>,
}

pub struct Discussion {
    pub comments: Vec<Comment>,
    pub count: usize,
}

pub struct Comment {
    pub id: String,
    pub url: String,
    pub author: String,
    pub author_url: Option<String>,
    pub avatar: Option<String>,
    pub html: String,
    pub created: String,
    pub created_date: String,
    pub edited: bool,
    pub replies: Vec<Self>,
}

pub fn load<'a>(
    config: Option<&Giscus>,
    terms: impl IntoIterator<Item = &'a str>,
) -> Result<CommentSnapshot> {
    let Some(config) = config else {
        return Ok(CommentSnapshot::default());
    };
    let Ok(token) = env::var("GITHUB_TOKEN") else {
        return Ok(CommentSnapshot::default());
    };
    let token = token.trim();
    if token.is_empty() {
        return Ok(CommentSnapshot::default());
    }

    let terms = terms.into_iter().collect::<HashSet<_>>();
    let (owner, name) = config.repo.split_once('/').unwrap();
    let github = GitHub { token, owner, name };
    let discussions = github.discussions(&config.category_id, &terms)?;

    Ok(CommentSnapshot {
        discussions: Some(discussions),
    })
}

impl CommentSnapshot {
    pub const fn available(&self) -> bool {
        self.discussions.is_some()
    }

    pub fn discussion(&self, term: &str) -> Option<&Discussion> {
        self.discussions.as_ref()?.get(term)
    }
}

struct GitHub<'a> {
    token: &'a str,
    owner: &'a str,
    name: &'a str,
}

impl GitHub<'_> {
    fn discussions(
        &self,
        category_id: &str,
        terms: &HashSet<&str>,
    ) -> Result<HashMap<String, Discussion>> {
        let mut after = None;
        let mut discussions = HashMap::new();

        loop {
            let repository = self.request_repository::<ApiDiscussionList>(
                DISCUSSIONS_QUERY,
                &json!({
                    "owner": self.owner,
                    "name": self.name,
                    "categoryId": category_id,
                    "after": after.as_deref(),
                }),
            )?;
            let page = repository.discussions;
            for discussion in page
                .nodes
                .into_iter()
                .flatten()
                .filter(|discussion| terms.contains(discussion.title.as_str()))
            {
                let title = discussion.title.clone();
                let snapshot = self.discussion(discussion.number)?;
                if discussions.insert(title.clone(), snapshot).is_some() {
                    bail!("giscus category contains more than one discussion titled `{title}`");
                }
            }
            if !page.page_info.has_next_page {
                break;
            }
            after = page.page_info.end_cursor;
        }

        Ok(discussions)
    }

    fn discussion(&self, number: u64) -> Result<Discussion> {
        let mut after = None;
        let mut comments = Vec::new();

        loop {
            let repository = self.request_repository::<ApiDiscussionLookup>(
                COMMENTS_QUERY,
                &json!({
                    "owner": self.owner,
                    "name": self.name,
                    "number": number,
                    "after": after.as_deref(),
                }),
            )?;
            let thread = repository
                .discussion
                .with_context(|| format!("GitHub discussion #{number} was not found"))?;
            let page = thread.comments;
            for comment in page.nodes.into_iter().flatten() {
                if let Some(comment) = self.comment(comment)? {
                    comments.push(comment);
                }
            }
            if !page.page_info.has_next_page {
                break;
            }
            after = page.page_info.end_cursor;
        }

        let count = comments
            .iter()
            .map(|comment| 1 + comment.replies.len())
            .sum();

        Ok(Discussion { comments, count })
    }

    fn comment(&self, mut comment: ApiComment) -> Result<Option<Comment>> {
        if comment.is_minimized {
            return Ok(None);
        }

        let mut replies = comment
            .replies
            .take()
            .expect("the top-level comment query always includes replies");
        while replies.page_info.has_next_page {
            let data = self.request::<NodeData<ApiReplyList>>(
                REPLIES_QUERY,
                &json!({
                    "id": &comment.id,
                    "after": replies.page_info.end_cursor.as_deref(),
                }),
            )?;
            let node = data
                .node
                .context("GitHub comment disappeared while loading its replies")?;
            let page = node.replies;
            replies.nodes.extend(page.nodes);
            replies.page_info = page.page_info;
        }

        let replies = replies
            .nodes
            .into_iter()
            .flatten()
            .filter(|reply| !reply.is_minimized)
            .map(|reply| Comment::from_api(reply, Vec::new()))
            .collect();

        Ok(Some(Comment::from_api(comment, replies)))
    }

    fn request_repository<T>(&self, query: &str, variables: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.request::<RepositoryData<T>>(query, variables)?;

        response.repository.with_context(|| {
            format!(
                "GitHub repository {}/{} was not found",
                self.owner, self.name
            )
        })
    }

    fn request<T>(&self, query: &str, variables: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut response = ureq::post(GITHUB_GRAPHQL_URL)
            .header("Authorization", &format!("Bearer {}", self.token))
            .header("User-Agent", crate::GENERATOR)
            .send_json(json!({ "query": query, "variables": variables }))
            .with_context(|| {
                format!(
                    "failed to request giscus comments from {}/{}",
                    self.owner, self.name
                )
            })?;
        let response = response
            .body_mut()
            .read_json::<GraphQlResponse<T>>()
            .context("failed to decode the GitHub GraphQL response")?;
        if !response.errors.is_empty() {
            let messages = response
                .errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; ");
            bail!("GitHub GraphQL request failed: {messages}");
        }

        response
            .data
            .context("GitHub GraphQL response contained no data")
    }
}

impl Comment {
    fn from_api(comment: ApiComment, replies: Vec<Self>) -> Self {
        let created_date = comment.created_at[..10].to_owned();
        let (author, author_url, avatar) = match comment.author {
            Some(author) => (author.login, Some(author.url), Some(author.avatar_url)),
            None => ("ghost".to_owned(), None, None),
        };

        Self {
            id: comment.id,
            url: comment.url,
            author,
            author_url,
            avatar,
            html: comment.body_html,
            created: comment.created_at,
            created_date,
            edited: comment.last_edited_at.is_some(),
            replies,
        }
    }
}

#[derive(Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Deserialize)]
struct RepositoryData<T> {
    repository: Option<T>,
}

#[derive(Deserialize)]
struct NodeData<T> {
    node: Option<T>,
}

#[derive(Deserialize)]
struct ApiDiscussionList {
    discussions: Connection<ApiDiscussion>,
}

#[derive(Deserialize)]
struct ApiDiscussionLookup {
    discussion: Option<ApiThread>,
}

#[derive(Deserialize)]
struct ApiReplyList {
    replies: Connection<ApiComment>,
}

#[derive(Deserialize)]
struct ApiThread {
    comments: Connection<ApiComment>,
}

#[derive(Deserialize)]
struct Connection<T> {
    nodes: Vec<Option<T>>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct ApiDiscussion {
    number: u64,
    title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiComment {
    id: String,
    url: String,
    body_html: String,
    created_at: String,
    last_edited_at: Option<String>,
    is_minimized: bool,
    author: Option<ApiAuthor>,
    replies: Option<Connection<Self>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiAuthor {
    login: String,
    avatar_url: String,
    url: String,
}

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

const DISCUSSIONS_QUERY: &str = r#"
query($owner: String!, $name: String!, $categoryId: ID!, $after: String) {
  repository(owner: $owner, name: $name) {
    discussions(first: 100, after: $after, categoryId: $categoryId) {
      pageInfo { hasNextPage endCursor }
      nodes { number title }
    }
  }
}
"#;

const COMMENTS_QUERY: &str = r#"
query($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) {
    discussion(number: $number) {
      comments(first: 100, after: $after) {
        pageInfo { hasNextPage endCursor }
        nodes {
          id
          url
          bodyHtml: bodyHTML
          createdAt
          lastEditedAt
          isMinimized
          author { login avatarUrl url }
          replies(first: 100) {
            pageInfo { hasNextPage endCursor }
            nodes {
              id
              url
              bodyHtml: bodyHTML
              createdAt
              lastEditedAt
              isMinimized
              author { login avatarUrl url }
            }
          }
        }
      }
    }
  }
}
"#;

const REPLIES_QUERY: &str = r#"
query($id: ID!, $after: String) {
  node(id: $id) {
    ... on DiscussionComment {
      replies(first: 100, after: $after) {
        pageInfo { hasNextPage endCursor }
        nodes {
          id
          url
          bodyHtml: bodyHTML
          createdAt
          lastEditedAt
          isMinimized
          author { login avatarUrl url }
        }
      }
    }
  }
}
"#;
