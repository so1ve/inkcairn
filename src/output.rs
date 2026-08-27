use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::git::GitIndex;
use crate::url_path;

pub struct OutputFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub source: Option<PathBuf>,
}

pub struct SiteOutput<'a> {
    root: &'a Path,
    files: Vec<OutputFile>,
    git: Option<&'a GitIndex>,
}

#[derive(Serialize)]
struct ManifestSource<'a> {
    commit: Option<&'a str>,
    remote: Option<&'a str>,
    dirty: bool,
}

#[derive(Serialize)]
struct ManifestPage {
    path: String,
    source: Option<String>,
    hash: String,
}

#[derive(Serialize)]
struct Manifest<'a> {
    generator: &'static str,
    source: ManifestSource<'a>,
    pages: Vec<ManifestPage>,
}

impl<'a> SiteOutput<'a> {
    pub const fn new(root: &'a Path, files: Vec<OutputFile>, git: Option<&'a GitIndex>) -> Self {
        Self { root, files, git }
    }

    pub fn push(&mut self, file: OutputFile) {
        self.files.push(file);
    }

    pub fn html_paths(&self) -> Vec<String> {
        let mut paths = self
            .files
            .iter()
            .filter(|file| file.path.extension().unwrap() == "html")
            .map(|file| {
                let path = path_to_url(&file.path);

                if path == "index.html" {
                    String::new()
                } else {
                    url_path::encode(&path)
                }
            })
            .collect::<Vec<_>>();
        paths.sort();

        paths
    }

    fn copy_assets(&mut self) -> Result<()> {
        let directory = self.root.join("assets");
        let mut files = Vec::new();
        collect_files(&directory, &mut files)?;
        files.sort();

        for source in files {
            let relative = source.strip_prefix(&directory).unwrap();
            let bytes = fs::read(&source)?;

            self.files.push(OutputFile {
                path: PathBuf::from("assets").join(relative),
                bytes,
                source: Some(source.strip_prefix(self.root).unwrap().to_owned()),
            });
        }

        Ok(())
    }

    fn render_manifest(&self) -> OutputFile {
        let mut pages = self
            .files
            .iter()
            .map(|file| ManifestPage {
                path: path_to_url(&file.path),
                source: file.source.as_deref().map(path_to_url),
                hash: blake3::hash(&file.bytes).to_hex().to_string(),
            })
            .collect::<Vec<_>>();

        pages.sort_by(|left, right| left.path.cmp(&right.path));

        let manifest = Manifest {
            generator: crate::GENERATOR,
            source: ManifestSource {
                commit: self.git.and_then(|git| git.head.as_deref()),
                remote: self.git.and_then(|git| git.origin.as_deref()),
                dirty: self.git.is_some_and(|git| git.dirty),
            },
            pages,
        };

        let mut bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        bytes.push(b'\n');

        OutputFile {
            path: PathBuf::from("manifest.json"),
            bytes,
            source: None,
        }
    }

    fn emit(&self, temporary: &Path, destination: &Path) -> Result<()> {
        for file in &self.files {
            let destination = temporary.join(&file.path);

            fs::create_dir_all(destination.parent().unwrap())?;
            fs::write(&destination, &file.bytes)?;
        }

        match fs::remove_dir_all(destination) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        fs::rename(temporary, destination)?;

        Ok(())
    }

    pub fn write(mut self, destination: &Path) -> Result<PathBuf> {
        self.copy_assets()?;

        let manifest = self.render_manifest();
        self.files.push(manifest);

        let mut paths = HashSet::new();
        for file in &self.files {
            if !paths.insert(&file.path) {
                bail!("more than one page resolves to {}", file.path.display());
            }
        }

        let parent = destination.parent().unwrap();
        let name = destination.file_name().unwrap().to_str().unwrap();
        let temporary = parent.join(format!(".{name}-build-{}", std::process::id()));
        fs::create_dir_all(parent)?;
        fs::create_dir(&temporary)?;

        match self.emit(&temporary, destination) {
            Ok(()) => {}
            Err(error) => {
                _ = fs::remove_dir_all(&temporary);

                return Err(error);
            }
        }

        Ok(destination.to_owned())
    }
}

fn collect_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_files(&path, output)?;
        } else if file_type.is_file() {
            output.push(path);
        }
    }

    Ok(())
}

fn path_to_url(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "/")
}
