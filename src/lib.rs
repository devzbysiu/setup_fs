use doc_comment::doctest;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, char, space0, space1};
use nom::combinator::opt;
use nom::multi::many1;
use nom::sequence::tuple;
use nom::IResult;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use thiserror::Error;

doctest!("../README.md");

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot create dir")]
    Fs(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::missing_errors_doc)]
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

#[allow(dead_code)]
fn parse_tree(input: &str) -> IResult<&str, Tree> {
    let (i, subtrees) = many1(subtree)(input)?;
    Ok((i, Tree::new(subtrees)))
}

fn subtree(input: &str) -> IResult<&str, Subtree> {
    let (i, (root, mut entries)) = tuple((root, many1(entry)))(input)?;
    entries.insert(0, root);
    Ok((i, Subtree::new(entries)))
}

fn root(input: &str) -> IResult<&str, Entry> {
    let (i, (_, root, _)) = tuple((tag("|_"), alpha1, char('\n')))(input)?;
    Ok((i, Entry::new(root)))
}

fn entry(input: &str) -> IResult<&str, Entry> {
    let (i, (_, value, _)) = tuple((entry_prefix, alpha1, char('\n')))(input)?;
    Ok((i, Entry::new(value)))
}

fn entry_prefix(input: &str) -> IResult<&str, ()> {
    let (i, _) = tuple((space1, opt(tuple((char('|'), space1))), tag("|_")))(input)?;
    Ok((i, ()))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Tree {
    subtrees: Vec<Subtree>,
}

impl Tree {
    fn new(subtrees: Vec<Subtree>) -> Self {
        Self {
            subtrees: subtrees.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Subtree {
    entries: Vec<Entry>,
}

impl Subtree {
    fn new(entries: Vec<Entry>) -> Self {
        Self {
            entries: entries.to_vec(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct Entry {
    value: String,
}

impl Entry {
    fn new<S: Into<String>>(value: S) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;
    use tempfile::TempDir;

    #[test]
    fn test_root() {
        // given
        let input = r#"|_testentry
"#;

        // when
        let res = root(input);

        // then
        assert_eq!(res, Ok(("", Entry::new("testentry"))));
    }

    #[test]
    fn test_entry_when_many_subtrees() {
        // given
        let input = r#"| |_testentry
"#;

        // when
        let res = entry(input);

        // then
        assert_eq!(res, Ok(("", Entry::new("testentry"))));
    }

    #[test]
    fn test_subtree_without_file_content() {
        let tree = r#"|_initialcontent
                        |_jcrroot
                          |_content
                            |_testfile
"#;

        // when
        let res = subtree(tree);

        // then
        assert_eq!(
            res,
            Ok((
                "",
                Subtree::new(vec![
                    Entry::new("initialcontent"),
                    Entry::new("jcrroot"),
                    Entry::new("content"),
                    Entry::new("testfile")
                ])
            ))
        );
    }

    #[test]
    fn test_tree_with_single_subtree() {
        let tree = r#"|_initialcontent
                        |_jcrroot
                          |_content
                            |_testfile
"#;

        // when
        let res = parse_tree(tree);

        // then
        assert_eq!(
            res,
            Ok((
                "",
                Tree::new(vec![Subtree::new(vec![
                    Entry::new("initialcontent"),
                    Entry::new("jcrroot"),
                    Entry::new("content"),
                    Entry::new("testfile")
                ]),])
            ))
        );
    }

    #[test]
    fn test_parse_tree_with_multiple_subtrees() {
        let tree = r#"|_initialcontent
                      | |_jcrroot
                      |   |_content
                      |     |_testfile
                      |_otherdir
                        |_subdir
                          |_testfile
"#;
        //         let tree = r#"|_initialcontent
        //                         |_jcrroot
        // "#;

        // when
        let res = parse_tree(tree);

        // then
        assert_eq!(
            res,
            Ok((
                "",
                Tree::new(vec![
                    Subtree::new(vec![Entry::new("initialcontent"), Entry::new("jcrroot"),]),
                    // Subtree::new(vec![
                    //     Entry::new("otherdir"),
                    //     Entry::new("subdir"),
                    //     Entry::new("testfile"),
                    // ])
                ])
            ))
        );
    }

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
