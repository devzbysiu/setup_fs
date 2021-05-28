use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

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
        let parts: Vec<&str> = chars.split("\"").collect();
        let path = PathBuf::from(parts[0]);
        let filecontent = parts[1].to_string();
        res.push((path, filecontent));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_fs_tree() -> Result<()> {
        let tree = r#"
        |_initial-content
        | |_jcr-root
        |   |_content
        |     "initial-content"
        |_server-zip
          |_jcr-root
            |_content
              "zip-content"
    "#;

        let files = parse_fs_tree(tree);
        assert_eq!(
            files[0],
            (
                PathBuf::from("/initial-content/jcr-root/content"),
                "initial-content".into()
            )
        );
        Ok(())
    }
}
