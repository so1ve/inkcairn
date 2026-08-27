use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use time::Date;
use time::macros::format_description;

pub struct FileInfo {
    pub created_at: Date,
    pub updated_at: Date,
    pub dirty: bool,
}

struct Repository {
    root: PathBuf,
}

impl Repository {
    fn discover(directory: &Path) -> io::Result<Option<Self>> {
        let output = match Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if !output.status.success() {
            return Ok(None);
        }
        let root = String::from_utf8(output.stdout).unwrap();

        Ok(Some(Self {
            root: std::fs::canonicalize(root.trim())?,
        }))
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root);

        command
    }
}

pub struct GitIndex {
    repository: Repository,
    pub head: Option<String>,
    pub origin: Option<String>,
    pub dirty: bool,
}

impl GitIndex {
    pub fn discover(site_root: &Path) -> io::Result<Option<Self>> {
        let Some(repository) = Repository::discover(site_root)? else {
            return Ok(None);
        };
        let status = repository
            .command()
            .args(["status", "--porcelain=v2", "--branch"])
            .output()?;
        if !status.status.success() {
            return Err(io::Error::other("Git could not read repository status"));
        }
        let status = String::from_utf8(status.stdout).unwrap();
        let oid = status
            .lines()
            .find_map(|line| line.strip_prefix("# branch.oid "))
            .unwrap();
        let head = if oid == "(initial)" {
            None
        } else {
            Some(oid.to_owned())
        };
        let dirty = status.lines().any(|line| !line.starts_with("# "));
        let origin = repository
            .command()
            .args(["remote", "get-url", "origin"])
            .output()?;
        let origin = if origin.status.success() {
            remote_url(String::from_utf8(origin.stdout).unwrap().trim())
        } else {
            None
        };

        Ok(Some(Self {
            repository,
            head,
            origin,
            dirty,
        }))
    }

    pub fn file_info(&self, path: &Path) -> io::Result<Option<FileInfo>> {
        let relative = path.strip_prefix(&self.repository.root).unwrap();
        let status = self
            .repository
            .command()
            .args(["status", "--porcelain", "--"])
            .arg(relative)
            .output()?;
        if !status.status.success() {
            return Err(io::Error::other("Git could not inspect content status"));
        }
        let dirty = !status.stdout.is_empty();
        let output = self
            .repository
            .command()
            .args(["log", "--follow", "--reverse", "--format=%cs"])
            .arg("--")
            .arg(relative)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("Git could not read content history"));
        }
        let output = String::from_utf8(output.stdout).unwrap();
        let mut dates = output.lines().filter(|line| !line.is_empty());
        let Some(created_at) = dates.next() else {
            return Ok(None);
        };
        let format = format_description!("[year]-[month]-[day]");
        let created_at = Date::parse(created_at, format).unwrap();
        let updated_at = match dates.next_back() {
            Some(date) => Date::parse(date, format).unwrap(),
            None => created_at,
        };

        Ok(Some(FileInfo {
            created_at,
            updated_at,
            dirty,
        }))
    }
}

fn remote_url(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches('/').trim_end_matches(".git");
    let (host, path) = if let Some(rest) = value.strip_prefix("git@") {
        rest.split_once(':')?
    } else if let Some(rest) = value.strip_prefix("ssh://git@") {
        rest.split_once('/')?
    } else {
        let rest = if let Some(rest) = value.strip_prefix("https://") {
            rest
        } else {
            value.strip_prefix("http://")?
        };
        rest.split_once('/')?
    };
    if !matches!(host, "github.com" | "gitlab.com" | "codeberg.org") || path.is_empty() {
        return None;
    }

    Some(format!("https://{host}/{path}"))
}
