#![deny(missing_docs)] // TODO: uncomment this
#![doc(html_root_url = "https://docs.rs/setup_fs/0.1.0")]

//! This library is very simple (or even primitive) way of setting up directories on the filesystem
//! by passing tree-like "description" of the filesystem.
//!
//! # Example
//!  ```
//! use tempfile::TempDir;
//! use std::error::Error;
//! use setup_fs::setup_fs;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let tree = r#"
//!           |_initial-content
//!           | |_jcr-root
//!           |   |_content
//!           |     |_test-file
//!           |       "initial-content"
//!           |_server-zip
//!             |_jcr-root
//!               |_content
//!                 |_test-file
//!                   "zip-content"
//!       "#;
//!     let tmp_dir = TempDir::new()?;
//!     setup_fs(tmp_dir.path(), tree)?;
//!     Ok(())
//! }
//! ```
//!
//! # **Warning**
//! This library is not production ready. Consider it as a quick way to setup the filesystem for
//! the testing purposes.

use doc_comment::doctest;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

doctest!("../README.md");

/// Error which can be returned by [setup_fs](setup_fs) function.
#[derive(Error, Debug)]
pub enum SetupFsError {
    /// Error was returned while creating a directory.
    #[error("cannot create dir '{0}': {1}")]
    DirCreation(PathBuf, String),

    /// Error was returned while creating a file.
    #[error("cannot create file '{0}': {1}")]
    FileCreation(PathBuf, String),

    /// Error was returned while writing to a file.
    #[error("cannot write to file '{0}': {1}")]
    WritingFile(PathBuf, String),

    /// Error was returned while creating full path. It can happen only when the root directory
    /// passed to the [setup_fs](setup_fs) was the root of the filesystem and the path in the tree
    /// was empty.
    #[error("cannot get parent directory of {0}")]
    EmptyPath(PathBuf),
}

/// Result type of the library.
pub type Result<T> = std::result::Result<T, SetupFsError>;

/// The one and only public function of this library. It first converts the input string to a list
/// of tuples `(PathBuf, String)`. The `String` here represents file content and it's not required.
/// Then for each tuple on the list it simply creates each file writes the content if it's
/// available.
#[allow(clippy::missing_errors_doc)]
pub fn setup_fs<P: AsRef<Path>, S: Into<String>>(root: P, tree: S) -> Result<()> {
    let entries = parse_fs_tree(tree);
    for (path, content) in entries {
        let full_path = root.as_ref().join(path);
        let dir = full_path
            .parent()
            .ok_or_else(|| SetupFsError::EmptyPath(root.as_ref().to_path_buf()))?;
        create_dir_all(dir)
            .map_err(|e| SetupFsError::DirCreation(dir.to_path_buf(), e.to_string()))?;
        let mut file = File::create(&full_path)
            .map_err(|e| SetupFsError::FileCreation(full_path.clone(), e.to_string()))?;
        file.write_all(content.as_bytes())
            .map_err(|e| SetupFsError::WritingFile(full_path, e.to_string()))?;
    }
    Ok(())
}

fn parse_fs_tree<S: Into<String>>(tree: S) -> Vec<(PathBuf, String)> {
    let mut res = Vec::new();
    let tree = tree
        .into()
        .replace("\n", "")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .replace("||", "|")
        .replace("|_", "/")
        .replace("|", "");
    let entries: Vec<&str> = tree.split_inclusive("\"/").collect();
    for entry in entries {
        let e = entry.replace("\"/", "\"");
        let mut chars = e.chars();
        chars.next_back();
        let chars = chars.as_str();
        let mut parts: Vec<&str> = chars.split('"').collect();
        let path_part = parts[0];
        if path_part.starts_with('/') {
            let mut part_chars = path_part.chars();
            part_chars.next();
            parts[0] = part_chars.as_str();
        }
        let path = PathBuf::from(parts[0]);
        let filecontent = parts[1].to_string();
        res.push((path, filecontent));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use tempfile::TempDir;

    #[test]
    fn test_parse_fs_tree() {
        // given
        let tree = r#"
        |_initial-content
        | |_jcr-root
        |   |_content
        |     |_test-file
        |       "initial-content"
        |
        |_server-zip
          |_jcr-root
            |_content
              |_test-file
                "zip-content"
    "#;

        // when
        let files = parse_fs_tree(tree);

        // then
        assert_eq!(files.len(), 2);
        assert_eq!(
            files[0],
            (
                PathBuf::from("initial-content/jcr-root/content/test-file"),
                "initial-content".into()
            )
        );

        assert_eq!(
            files[1],
            (
                PathBuf::from("server-zip/jcr-root/content/test-file"),
                "zip-content".into()
            )
        );
    }

    #[test]
    fn test_setup_fs() -> Result<()> {
        // given
        let tree = r#"
        |_initial-content
        | |_jcr-root
        |   |_content
        |     |_test-file
        |       "initial-content"
        |_server-zip
          |_jcr-root
            |_content
              |_test-file
                "zip-content"
    "#;
        let tmp_dir = TempDir::new().unwrap();

        // when
        setup_fs(tmp_dir.path(), tree)?;

        // then
        assert!(Path::new(
            &tmp_dir
                .path()
                .join("initial-content/jcr-root/content/test-file")
        )
        .exists());
        let content = read_to_string(
            &tmp_dir
                .path()
                .join("initial-content/jcr-root/content/test-file"),
        )
        .unwrap();
        assert_eq!(content, "initial-content");

        Ok(())
    }
}
