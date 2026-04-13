use crate::configs::DEFAULT_GITIGNORE_CONFIG_DIR;
use chrono::Local;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{error, warn};

#[derive(Debug, Deserialize)]
struct GitignoreConfig {
    name: String,
    description: String,
    enabled: bool,
    ignore: Option<Vec<String>>,
    sections: Option<std::collections::HashMap<String, SubSection>>,
}

#[derive(Debug, Deserialize)]
struct SubSection {
    enabled: bool,
    files: Option<Vec<String>>,
    files_disabled: Option<Vec<String>>,
}

fn parse_gitignore_config(source: &str, content: &str) -> Option<GitignoreConfig> {
    match toml::from_str(content) {
        Ok(config) => Some(config),
        Err(error) => {
            warn!("Skip invalid gitignore config {}: {}", source, error);
            None
        }
    }
}

fn load_gitignore_configs(config_dir: Option<&Path>) -> io::Result<Vec<GitignoreConfig>> {
    if let Some(dir) = config_dir {
        let mut entries = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("toml"))
            .collect::<Vec<PathBuf>>();
        entries.sort();

        return Ok(entries
            .into_iter()
            .filter_map(|path| {
                let content = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(error) => {
                        warn!(
                            "Skip unreadable gitignore config {}: {}",
                            path.display(),
                            error
                        );
                        return None;
                    }
                };

                parse_gitignore_config(&path.display().to_string(), &content)
            })
            .collect());
    }

    let mut files = DEFAULT_GITIGNORE_CONFIG_DIR.files().collect::<Vec<_>>();
    files.sort_by_key(|file| file.path().to_string_lossy().into_owned());

    Ok(files
        .into_iter()
        .filter_map(|file| {
            let content = file.contents_utf8()?;
            parse_gitignore_config(&file.path().display().to_string(), content)
        })
        .collect())
}

pub fn generate_gitignore(config_dir: Option<&Path>, force: bool) -> io::Result<()> {
    const PATH: &str = ".gitignore";

    if Path::new(PATH).exists() && !force {
        warn!("Skip existing {}", PATH);
        return Ok(());
    }

    let configs = load_gitignore_configs(config_dir)?;
    let mut file = File::create(PATH)?;

    let now = Local::now();
    writeln!(file, "# generated on {}", now.format("%Y-%m-%d %H:%M:%S"))?;

    for config in configs {
        if !config.enabled {
            continue;
        }

        writeln!(file, "### {} ###", config.name)?;
        writeln!(file, "# {}", config.description)?;

        if let Some(ignore_list) = config.ignore {
            for line in ignore_list {
                writeln!(file, "{line}")?;
            }
        }

        if let Some(sections) = config.sections {
            let mut sections = sections.into_iter().collect::<Vec<_>>();
            sections.sort_by(|(left, _), (right, _)| left.cmp(right));

            for (section_name, section) in sections {
                if section.enabled {
                    writeln!(file, "# section: {section_name}")?;
                    if let Some(files) = section.files {
                        for file_path in files {
                            writeln!(file, "{file_path}")?;
                        }
                    } else {
                        error!("{section_name} is enabled, but `files` is None");
                    }
                } else if let Some(files) = section.files_disabled {
                    writeln!(file, "# section: {section_name}")?;
                    for file_path in files {
                        writeln!(file, "{file_path}")?;
                    }
                }
            }
        }

        writeln!(file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::load_gitignore_configs;

    #[test]
    fn loads_embedded_gitignore_configs() {
        let configs = load_gitignore_configs(None).expect("load embedded configs");

        assert_eq!(configs.len(), 4);
        assert!(configs.iter().any(|config| config.name == "STM32CubeMX"));
        assert!(
            configs
                .iter()
                .any(|config| config.name == "IDE and Editor files")
        );
    }
}
