use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot create dir")]
    Fs(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn setup_fs<P: AsRef<Path>, S: Into<String>>(root: P, tree: S) -> Result<()> {
    let entries = parse_fs_tree(tree);
    for (path, content) in entries {
        let full_path = root.as_ref().join(path);
        let dir = full_path.parent().expect("not supported");
        create_dir_all(dir)?;
        let mut file = File::create(&full_path)?;
        file.write_all(content.as_bytes())?;
    }
    Ok(())
}

fn parse_fs_tree<S: Into<String>>(tree: S) -> Vec<(PathBuf, String)> {
    let mut res = Vec::new();
    let mut tree = tree.into();
    tree = tree.replace("\n", "");
    tree = tree.chars().filter(|c| !c.is_whitespace()).collect();
    tree = tree.replace("||", "|");
    tree = tree.replace("|_", "/");
    tree = tree.replace("|", "");
    let entries: Vec<&str> = tree.split_inclusive("\"/").collect();
    for entry in entries {
        let e = entry.replace("\"/", "\"");
        let mut chars = e.chars();
        chars.next_back();
        let chars = chars.as_str();
        let mut parts: Vec<&str> = chars.split("\"").collect();
        let path_part = parts[0];
        if path_part.chars().next() == Some('/') {
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
    fn test_parse_fs_tree() -> Result<()> {
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
        Ok(())
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
        let tmp_dir = TempDir::new()?;

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
        )?;
        assert_eq!(content, "initial-content");

        Ok(())
    }
}
