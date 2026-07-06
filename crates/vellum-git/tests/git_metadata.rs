use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use vellum_git::{RemoteOrigin, Repo};

// ── Remote origin ──

#[test]
fn parse_github_gitlab_codeberg() {
    let gh = RemoteOrigin::parse("git@github.com:so1ve/vellum.git").unwrap();
    assert_eq!(gh.web_url, "https://github.com/so1ve/vellum");
    assert_eq!(
        gh.commit_url("abc"),
        "https://github.com/so1ve/vellum/commit/abc"
    );

    let gl = RemoteOrigin::parse("https://gitlab.com/group/subgroup/blog.git").unwrap();
    assert_eq!(gl.web_url, "https://gitlab.com/group/subgroup/blog");
    assert_eq!(
        gl.commit_url("def"),
        "https://gitlab.com/group/subgroup/blog/-/commit/def"
    );

    let cb = RemoteOrigin::parse("ssh://git@codeberg.org/miku/notes.git").unwrap();
    assert_eq!(cb.web_url, "https://codeberg.org/miku/notes");
    assert_eq!(
        cb.commit_url("789"),
        "https://codeberg.org/miku/notes/commit/789"
    );

    assert!(RemoteOrigin::parse("https://example.com/owner/repo.git").is_none());
}

#[test]
fn parse_origin_edge_cases() {
    let gh = RemoteOrigin::parse("https://github.com:443/so1ve/vellum.git").unwrap();
    assert_eq!(gh.web_url, "https://github.com/so1ve/vellum");

    let gl = RemoteOrigin::parse("ssh://git@gitlab.com:22/group/blog.git/").unwrap();
    assert_eq!(gl.web_url, "https://gitlab.com/group/blog");

    assert!(RemoteOrigin::parse("git@github.com:owner.git").is_none());
    assert!(RemoteOrigin::parse("https://github.com:443/owner").is_none());
}

// ── Repo state ──

#[test]
fn state_head_origin_dirty() {
    let repo = TestRepo::new("state");
    repo.init();
    repo.run(["remote", "add", "origin", "git@github.com:so1ve/vellum.git"]);
    repo.write("posts/hello.md", "# Hello\n");
    repo.run(["add", "."]);
    repo.run(["commit", "-m", "initial"]);
    repo.write("draft.md", "untracked\n");

    let r = Repo::open(repo.path()).unwrap();
    let s = r.state().unwrap();
    assert_eq!(s.head.len(), 40);
    assert_eq!(s.short_head, s.head[..7]);
    assert!(s.dirty);
    let o = s.origin.unwrap();
    assert_eq!(
        o.commit_url(&s.head),
        format!("https://github.com/so1ve/vellum/commit/{}", s.head)
    );
}

// ── File info ──

#[test]
fn file_info_created_and_updated() {
    let repo = TestRepo::new("file");
    repo.init();
    repo.write("posts/hello.md", "# Hello\n");
    repo.run(["add", "."]);
    repo.run(["commit", "-m", "create"]);

    repo.write("posts/hello.md", "# Hello\n\nUpdated.\n");
    repo.run(["add", "."]);
    repo.run(["commit", "-m", "update"]);

    let r = Repo::open(repo.path()).unwrap();
    let info = r.file_info("posts/hello.md").unwrap().unwrap();
    assert_eq!(info.created_commit.len(), 40);
    assert_eq!(info.updated_commit.len(), 40);
    assert_ne!(info.created_commit, info.updated_commit);
    assert!(info.created_at.contains('T'));
    assert!(info.updated_at.contains('T'));
}

#[test]
fn file_info_nonexistent() {
    let repo = TestRepo::new("none");
    repo.init();
    repo.write("exists.md", "x\n");
    repo.run(["add", "."]);
    repo.run(["commit", "-m", "init"]);

    let r = Repo::open(repo.path()).unwrap();
    assert!(r.file_info("nope.md").unwrap().is_none());
}

#[test]
fn file_info_brackets_and_absolute() {
    let repo = TestRepo::new("brackets");
    repo.init();
    repo.write("posts/[hello].md", "# Hello\n");
    repo.run(["add", "."]);
    repo.run(["commit", "-m", "create"]);

    let r = Repo::open(repo.path()).unwrap();
    let rel = r.file_info("posts/[hello].md").unwrap().unwrap();
    let abs = r
        .file_info(repo.path().join("posts/[hello].md"))
        .unwrap()
        .unwrap();
    assert_eq!(rel.created_commit, abs.created_commit);
    assert_eq!(rel.updated_commit, abs.updated_commit);
}

// ── Harness ──

struct TestRepo {
    path: PathBuf,
}

impl TestRepo {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("vellum_git_{name}_{stamp}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn init(&self) {
        self.run(["init"]);
        self.run(["config", "user.email", "vellum@test.invalid"]);
        self.run(["config", "user.name", "Vellum"]);
        self.run(["config", "commit.gpgsign", "false"]);
    }

    fn write(&self, relative: &str, content: &str) {
        let p = self.path.join(relative);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, content).unwrap();
    }

    fn run<const N: usize>(&self, args: [&str; N]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed — {}",
            String::from_utf8_lossy(&out.stderr),
        );
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&self.path);
    }
}
