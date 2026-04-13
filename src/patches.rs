use regex::Regex;
use serde::Deserialize;
use std::fs;
use tracing::debug;

#[derive(Debug, Deserialize)]
#[serde(tag = "mode")]
pub enum Patch {
    #[serde(rename = "append")]
    Append {
        file: String,
        after: String,
        insert: String,
        marker: String,
    },
    #[serde(rename = "replace")]
    Replace {
        file: String,
        find: String,
        insert: String,
    },
    #[serde(rename = "regex_replace")]
    RegexReplace {
        file: String,
        pattern: String,
        insert: String,
    },
    #[serde(rename = "uncomment_block")]
    UncommentBlock { file: String, marker: String },
}

pub fn apply_patch(patch: &Patch) -> std::io::Result<()> {
    let file = get_file(patch);
    debug!(file, ?patch, "Applying file patch");

    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(error) => {
            debug!(file, %error, "Skipping patch because target file is unavailable");
            return Ok(());
        }
    };

    let new_content = match patch {
        Patch::Append {
            after,
            insert,
            marker,
            ..
        } => {
            if content.contains(marker) {
                debug!(
                    file,
                    marker, "Skipping append patch because marker already exists"
                );
                return Ok(());
            }
            content
                .lines()
                .map(|line| {
                    if line.contains(after) {
                        format!("{line}\n{insert}")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
        Patch::Replace { find, insert, .. } => {
            if content.contains(insert) {
                debug!(
                    file,
                    insert, "Skipping replace patch because target content already exists"
                );
                return Ok(());
            }
            content.replace(find, insert)
        }
        Patch::RegexReplace {
            pattern, insert, ..
        } => {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&content) && content.contains(insert) {
                debug!(
                    file,
                    pattern, "Skipping regex patch because replacement already exists"
                );
                return Ok(());
            }
            re.replace_all(&content, insert.as_str()).to_string()
        }
        Patch::UncommentBlock { marker, .. } => {
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

            let mut in_block = false;
            for line in &mut lines {
                if line.contains(marker) {
                    in_block = true;
                    continue;
                }

                if in_block {
                    if line.starts_with('#') {
                        *line = line.trim_start_matches('#').trim().to_string();
                    } else {
                        break;
                    }
                }
            }

            lines.join("\n") + "\n"
        }
    };

    if new_content == content {
        debug!(file, "Patch produced no textual changes");
        return Ok(());
    }

    debug!(file, bytes = new_content.len(), "Writing patched file");
    fs::write(file, new_content)?;
    Ok(())
}

fn get_file(patch: &Patch) -> &str {
    match patch {
        Patch::Append { file, .. } => file,
        Patch::Replace { file, .. } => file,
        Patch::RegexReplace { file, .. } => file,
        Patch::UncommentBlock { file, .. } => file,
    }
}
