use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};

const POST: &str =
    "# Hello, world!\n\nWelcome to my new blog.\n\n## First post\n\nStart writing here.\n";

pub fn run(requested_root: &Path) -> Result<PathBuf> {
    if requested_root.exists() {
        for entry in fs::read_dir(requested_root)? {
            if entry?.file_name() != ".git" {
                bail!("{} is not empty", requested_root.display());
            }
        }
    } else {
        fs::create_dir_all(requested_root)?;
    }

    let root = fs::canonicalize(requested_root)?;
    let new_repository = !root.join(".git").exists();
    let title = root.file_name().unwrap().to_str().unwrap();

    fs::create_dir(root.join("posts"))?;
    fs::write(root.join("inkcairn.md"), format!("# {title}\n"))?;
    fs::write(root.join("posts/hello-world.md"), POST)?;
    fs::write(root.join(".gitignore"), "dist/\n")?;

    if new_repository {
        let initialized = Command::new("git")
            .args(["init", "--quiet"])
            .arg(&root)
            .status()?;
        if !initialized.success() {
            bail!("Git could not initialize {}", root.display());
        }

        let staged = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["add", "--all"])
            .status()?;
        if !staged.success() {
            bail!("Git could not stage the initialized site");
        }

        let committed = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["commit", "--quiet", "-m", "Initialize Inkcairn site"])
            .status()?;
        if !committed.success() {
            bail!("Git could not create the initial commit");
        }
    }

    Ok(root)
}
