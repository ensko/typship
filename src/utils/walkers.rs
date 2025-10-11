use std::path::Path;

use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::{Walk, WalkBuilder};

use super::read_manifest;

/// Only `.typstignore`
pub fn walker_publish(root: &Path) -> Walk {
    WalkBuilder::new(root)
        .standard_filters(true)
        .add_custom_ignore_filename(".typstignore")
        .build()
}

/// `.typstignore` and `package.excludes`
pub fn walker_install(
    root: &Path,
) -> Result<Vec<std::result::Result<ignore::DirEntry, ignore::Error>>> {
    let manifest = read_manifest(root)?;

    // based on https://github.com/typst/packages/blob/84765d25df36f59425384e37f6ae70c62fe1c3df/bundler/src/main.rs#L331-L343
    // license: Apache-2.0
    let mut overrides = OverrideBuilder::new(root);
    for exclusion in &manifest.package.exclude {
        let exclusion = exclusion.trim_start_matches("./");
        overrides
            .add(&format!("!{exclusion}"))
            .context("failed to process exclude pattern")?;
    }
    // Always ignore the thumbnail.
    if let Some(template) = &manifest.template {
        if let Some(thumbnail) = &template.thumbnail {
            overrides
                .add(&format!("!{thumbnail}"))
                .context("failed to process exclude pattern")?;
        }
    }
    let overrides = overrides
        .build()
        .context("failed to process exclude patterns")?;

    Ok(WalkBuilder::new(root)
        .standard_filters(true)
        .add_custom_ignore_filename(".typstignore")
        .overrides(overrides)
        .build()
        .map(|entry| {
            if let Ok(path) = entry {
                return Ok(path);
            }
            Err(entry.unwrap_err())
        })
        .collect())
}
