use std::{
    fs,
    path::{Path, PathBuf},
};

use include_dir::Dir;
use indexmap::IndexMap;

use crate::{app::RepoBuilder, prelude::*};

/// Recursively copies `src` into `dst`, creating `dst` and any missing parents.
///
/// `read_dir` yields dot-entries, so hidden files and directories are included.
///
/// # Errors
///
/// Returns an error if `dst` cannot be created, `src` cannot be read, or any
/// file copy fails.
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Recursively walks `dir`, inserting every file it finds into `entries` keyed
/// by its path relative to `root`.
///
/// Returns `None` if any directory read, file read, or prefix strip fails —
/// notably, a non-UTF-8 file aborts the whole walk rather than being skipped.
pub fn collect_files(
    root: &Path,
    dir: &Path,
    entries: &mut IndexMap<String, (String, PathBuf)>,
) -> Option<()> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let ty = entry.file_type().ok()?;
        if ty.is_dir() {
            collect_files(root, &entry.path(), entries)?; // recurse
        } else if ty.is_file() {
            let content = fs::read_to_string(entry.path()).ok()?;
            let key = entry
                .path()
                .strip_prefix(root)
                .ok()?
                .to_string_lossy()
                .into_owned();
            entries.insert(key, (content, entry.path()));
        }
    }
    Some(())
}

/// Recursively writes every file embedded in `dir` under `root`.
///
/// `include_dir` paths are relative to the embedded root, so nesting is
/// reproduced as-is. UTF-8 files go through [`substitute`]; anything else is
/// copied byte-for-byte, which is what keeps binary templates intact.
///
/// # Errors
///
/// Returns an error if a directory cannot be created or a file cannot be
/// written.
pub fn write_dir(dir: &Dir<'_>, root: &Path, repo: &RepoBuilder) -> Result<()> {
    for file in dir.files() {
        let name = file.path().to_string_lossy();
        let bytes = match file.contents_utf8() {
            Some(text) => substitute(text, repo),
            None => file.contents().to_vec(),
        };
        write_entry(root, &name, &bytes)?;
    }
    for sub in dir.dirs() {
        write_dir(sub, root, repo)?;
    }
    Ok(())
}

/// Writes `content` to `root`/`name`, creating any missing parent directories.
///
/// # Errors
///
/// Returns an error if the parent directories cannot be created or the write
/// fails.
pub fn write_entry(root: &Path, name: &str, content: &[u8]) -> Result<()> {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(())
}

/// Marker opening a tool-conditional template block.
const IF_MARKER: &str = "{if:";
/// Marker closing a tool-conditional template block.
const ENDIF_MARKER: &str = "{endif:";
/// Marker filtering a tool-conditional template block.
const IFNOT_MARKER: &str = "{ifnot:";

/// Rewrites a template on its way out: `{if:<tool>}`/`{ifnot:<tool>}` blocks first, then the
/// `{name}`, `{desc}`, and `{owner}` placeholders.
///
/// Order matters — conditionals are resolved before placeholders, so a
/// `{name}` sitting inside a block belonging to an unselected tool is dropped
/// rather than substituted.
#[must_use]
pub fn substitute(content: &str, repo: &RepoBuilder) -> Vec<u8> {
    strip_conditionals(content, repo)
        .replace("{ident}", &ident(&repo.name))
        .replace("{name}", &repo.name)
        .replace("{desc}", &repo.desc)
        .replace("{owner}", &repo.owner)
        .into_bytes()
}

/// Extracts the tool name out of a `{<marker><tool>}` line, if/ifnot it has one.
///
/// The marker is matched anywhere in the line, so it can sit behind whatever
/// comment syntax the template's file format uses.
fn marker_tool<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let start = line.find(marker)? + marker.len();
    let end = start + line[start..].find('}')?;
    Some(line[start..end].trim())
}

/// Drops `{if:<tool>}`/`{ifnot:<tool>}`/`{endif:<tool>}` blocks whose tool is not selected,
/// keeping the body of the ones whose tool is. The marker lines themselves are
/// removed either way.
fn strip_conditionals(content: &str, repo: &RepoBuilder) -> String {
    if !content.contains(IF_MARKER) && !content.contains(IFNOT_MARKER) {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len());
    let mut skipping: Option<&str> = None;
    for line in content.lines() {
        if let Some(tool) = marker_tool(line, ENDIF_MARKER) {
            if skipping == Some(tool) {
                skipping = None;
            }
            continue;
        } else if let Some(tool) = marker_tool(line, IF_MARKER) {
            if skipping.is_none() && !repo.is_selected(tool) {
                skipping = Some(tool);
            }
            continue;
        } else if let Some(tool) = marker_tool(line, IFNOT_MARKER) {
            if skipping.is_none() && repo.is_selected(tool) {
                skipping = Some(tool);
            }
            continue;
        }
        if skipping.is_none() {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// `{name}` folded into a Rust identifier: lowercased, with every character
/// that is not ASCII alphanumeric replaced by `_`, and a leading `_` if the
/// result would start with a digit.
fn ident(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}
