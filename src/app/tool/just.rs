use std::path::Path;

use include_dir::{Dir, include_dir};

use crate::app::{RepoBuilder, Tool, tool::category::Category};
use crate::prelude::*;

/// Just file templates path.
static JUST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/just");

/// Task runner recipes: check, lint, fmt, ci.
#[derive(Debug)]
pub(crate) struct Just;

impl Tool for Just {
    fn name(&self) -> String {
        "justfile".to_string()
    }

    fn desc(&self) -> String {
        "Task runner recipes: check, lint, fmt, ci.".to_string()
    }

    fn category(&self) -> Category {
        Category::Build
    }

    fn default_setup(&self) -> bool {
        true
    }

    fn gen_template(&self, root: &Path, repo: &RepoBuilder) -> Result<()> {
        write_dir(&JUST, root, repo)?;
        Ok(())
    }
}
