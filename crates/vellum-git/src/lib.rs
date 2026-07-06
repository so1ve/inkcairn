use std::path::{Path, PathBuf};
use std::process::Command;

mod error;

fn git(dir: &Path, args: &[&str]) -> Result<String, error::Error> {
    let out = Command::new("git").args(args).current_dir(dir).output()?;
    if out.status.success() {
        return Ok(String::from_utf8(out.stdout)?);
    }

    Err(error::Error::Exit {
        args: args.iter().map(|a| a.to_string()).collect(),
        code: out.status.code(),
        stderr: String::from_utf8_lossy(&out.stderr).into(),
    })
}

fn git_ok(dir: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
}

fn deconstruct(remote: &str) -> Option<(String, &str)> {
    if let Some(rest) = remote.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;

        return Some((host.to_ascii_lowercase(), path));
    }
    let (_, rest) = remote.split_once("://")?;
    let (auth, path) = rest.split_once('/')?;
    let host = auth.rsplit_once('@').map(|(_, h)| h).unwrap_or(auth);
    let host = host
        .rsplit_once(':')
        .map(|(h, p)| {
            if p.bytes().all(|c| c.is_ascii_digit()) {
                h
            } else {
                host
            }
        })
        .unwrap_or(host);

    if host.is_empty() {
        None
    } else {
        Some((host.to_ascii_lowercase(), path))
    }
}

pub struct Repo {
    root: PathBuf,
}

impl Repo {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, error::Error> {
        let root = git(path.as_ref(), &["rev-parse", "--show-toplevel"])?;

        Ok(Self {
            root: PathBuf::from(root.trim()),
        })
    }

    pub fn state(&self) -> Result<RepoState, error::Error> {
        let head = self.run(&["rev-parse", "HEAD"])?;
        let head = head.trim().to_owned();
        let origin = self
            .run_ok(&["remote", "get-url", "origin"])
            .and_then(|u| RemoteOrigin::parse(u.trim()));
        let dirty = !self.run(&["status", "--porcelain"])?.is_empty();

        Ok(RepoState {
            head: head.clone(),
            short_head: head.chars().take(7).collect(),
            origin,
            dirty,
        })
    }

    pub fn file_info(&self, file: impl AsRef<Path>) -> Result<Option<FileInfo>, error::Error> {
        let rel = self.relative(file.as_ref());
        let out = self.run(&[
            "log",
            "--follow",
            "--format=%H%x00%cI",
            "--",
            &format!(":(literal){rel}"),
        ])?;
        let commits: Vec<(&str, &str)> = out.lines().filter_map(|l| l.split_once('\0')).collect();
        // Might be empty if the file is untracked or deleted, or if the path is invalid
        if commits.is_empty() {
            return Ok(None);
        }
        let (created_commit, created_at) = commits[0];
        let (updated_commit, updated_at) = commits[commits.len() - 1];

        Ok(Some(FileInfo {
            created_commit: created_commit.to_string(),
            created_at: created_at.to_string(),
            updated_commit: updated_commit.to_string(),
            updated_at: updated_at.to_string(),
        }))
    }

    fn relative(&self, file: &Path) -> String {
        let abs = if file.is_absolute() {
            file.to_path_buf()
        } else {
            self.root.join(file)
        };
        abs.strip_prefix(&self.root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn run(&self, args: &[&str]) -> Result<String, error::Error> {
        git(&self.root, args)
    }

    fn run_ok(&self, args: &[&str]) -> Option<String> {
        git_ok(&self.root, args)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoState {
    pub head: String,
    pub short_head: String,
    pub origin: Option<RemoteOrigin>,
    pub dirty: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileInfo {
    pub created_commit: String,
    pub created_at: String,
    pub updated_commit: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteOrigin {
    pub web_url: String,
    route: &'static str,
}

impl RemoteOrigin {
    /// Parse a remote URL. Supports GitHub, GitLab, Codeberg.
    #[must_use]
    pub fn parse(url: &str) -> Option<Self> {
        let (host, path) = deconstruct(url.trim())?;
        let route = match host.as_str() {
            "github.com" | "codeberg.org" => "commit",
            "gitlab.com" => "-/commit",
            _ => return None,
        };
        let path = path
            .trim_end_matches('/')
            .strip_suffix(".git")
            .unwrap_or(path.trim_end_matches('/'));
        if !path.contains('/') {
            return None;
        }

        Some(Self {
            web_url: format!("https://{host}/{path}"),
            route,
        })
    }

    #[must_use]
    pub fn commit_url(&self, hash: &str) -> String {
        format!("{}/{hash}", self.commit_prefix())
    }

    fn commit_prefix(&self) -> String {
        format!("{}/{}", self.web_url, self.route)
    }
}
