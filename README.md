<div align="center">

  <h1><code>setup_fs</code></h1>

  <h3>
    <strong>Easy way to create directory structures on the filesystem</strong>
  </h3>

  <p>
    <img src="https://img.shields.io/github/workflow/status/devzbysiu/setup_fs/ci?style=for-the-badge" alt="CI status badge" />
    <a href="https://codecov.io/gh/devzbysiu/setup_fs">
      <img src="https://img.shields.io/codecov/c/github/devzbysiu/setup_fs?style=for-the-badge&token=f2339b3de9e44be0a902458a669c1160" alt="Code coverage"/>
    </a>
    <img src="https://img.shields.io/badge/license-MIT%2FAPACHE--2.0-blue?style=for-the-badge" alt="License"/>
  </p>

  <h3>
    <a href="#about">About</a>
    <span> | </span>
    <a href="#example">Example</a>
    <span> | </span>
    <a href="#installation">Installation</a>
    <span> | </span>
    <a href="#license">License</a>
    <span> | </span>
    <a href="#contribution">Contribution</a>
  </h3>

  <sub><h4>Built with 🦀</h4></sub>
</div>

# <p id="about">About</p>

This is very small library which allows to quickly set up directory structures in tree-like manner.

**NOTE:** This is very limited, incomplete and not battle tested library. I'm using it in my
personal projects only in tests to quickly create desired filesystem without boilerplate.
**I don't recommend using it in production**.

# <p id="example">Example</p>

```rust
use tempfile::TempDir;
use std::error::Error;
use setup_fs::setup_fs;

fn main() -> Result<(), Box<dyn Error>> {
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
    setup_fs(tmp_dir.path(), tree)?;
    Ok(())
}
```

# <p id="installation">Installation</p>

Add as a dependency to your `Cargo.toml`:
```toml
[dependencies]
setup_fs = { git = "https://github.com/devzbysiu/setup_fs", rev = "036b2c8" }
```

# <p id="license">License</p>

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# <p id="contribution">Contribution</p>


Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
