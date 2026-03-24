use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::fs;
use std::path::Path;

use crate::options::{Options, SortOrder, ThousandsSeparator};

impl<'de> Deserialize<'de> for SortOrder {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SortOrderVisitor;

        impl<'de> de::Visitor<'de> for SortOrderVisitor {
            type Value = SortOrder;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a boolean or one of \"asc\", \"desc\", \"off\"")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<SortOrder, E> {
                Ok(if v { SortOrder::Asc } else { SortOrder::Off })
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<SortOrder, E> {
                v.parse().map_err(|msg: String| de::Error::custom(msg))
            }
        }

        deserializer.deserialize_any(SortOrderVisitor)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    pub indent: Option<usize>,
    pub currency_column: Option<usize>,
    pub cost_column: Option<usize>,
    pub thousands: Option<String>,
    pub spaces_in_braces: Option<bool>,
    pub fixed_cjk_width: Option<bool>,
    pub sort: Option<SortOrder>,
}

impl FileConfig {
    pub fn merge(self, other: FileConfig) -> FileConfig {
        FileConfig {
            indent: other.indent.or(self.indent),
            currency_column: other.currency_column.or(self.currency_column),
            cost_column: other.cost_column.or(self.cost_column),
            thousands: other.thousands.or(self.thousands),
            spaces_in_braces: other.spaces_in_braces.or(self.spaces_in_braces),
            fixed_cjk_width: other.fixed_cjk_width.or(self.fixed_cjk_width),
            sort: other.sort.or(self.sort),
        }
    }

    pub fn into_options(self) -> Options {
        let defaults = Options::default();
        let thousands_separator = self
            .thousands
            .as_deref()
            .map(|s| match s.to_ascii_lowercase().as_str() {
                "add" => ThousandsSeparator::Add,
                "remove" => ThousandsSeparator::Remove,
                _ => ThousandsSeparator::Keep,
            })
            .unwrap_or(defaults.thousands_separator);

        Options {
            indent: self.indent.unwrap_or(defaults.indent),
            currency_column: self.currency_column.unwrap_or(defaults.currency_column),
            cost_column: self.cost_column.unwrap_or(defaults.cost_column),
            thousands_separator,
            spaces_in_braces: self.spaces_in_braces.unwrap_or(defaults.spaces_in_braces),
            fixed_cjk_width: self.fixed_cjk_width.unwrap_or(defaults.fixed_cjk_width),
            sort: self.sort.unwrap_or(defaults.sort),
        }
    }
}

pub fn load_global() -> FileConfig {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            Path::new(&home).join(".config")
        });
    let path = config_dir.join("beanfmt").join("config.toml");
    load_file(&path)
}

pub fn find_project_config(start_dir: &Path) -> FileConfig {
    let mut dir = start_dir;
    loop {
        let dotfile = dir.join(".beanfmt.toml");
        if dotfile.is_file() {
            return load_file(&dotfile);
        }
        let plain = dir.join("beanfmt.toml");
        if plain.is_file() {
            return load_file(&plain);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return FileConfig::default(),
        }
    }
}

fn load_file(path: &Path) -> FileConfig {
    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("warning: failed to parse {}: {e}", path.display());
                FileConfig::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => FileConfig::default(),
        Err(e) => {
            eprintln!("warning: failed to read {}: {e}", path.display());
            FileConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_none_fields_preserved() {
        let base = FileConfig {
            indent: Some(2),
            currency_column: Some(60),
            ..FileConfig::default()
        };
        let overlay = FileConfig::default();
        let merged = base.merge(overlay);
        assert_eq!(merged.indent, Some(2));
        assert_eq!(merged.currency_column, Some(60));
    }

    #[test]
    fn merge_some_overrides() {
        let base = FileConfig {
            indent: Some(2),
            currency_column: Some(60),
            ..FileConfig::default()
        };
        let overlay = FileConfig {
            indent: Some(8),
            ..FileConfig::default()
        };
        let merged = base.merge(overlay);
        assert_eq!(merged.indent, Some(8));
        assert_eq!(merged.currency_column, Some(60));
    }

    #[test]
    fn into_options_fills_defaults() {
        let config = FileConfig {
            indent: Some(2),
            ..FileConfig::default()
        };
        let opts = config.into_options();
        assert_eq!(opts.indent, 2);
        assert_eq!(opts.currency_column, 70);
    }

    #[test]
    fn load_from_string_valid_toml() {
        let toml_str = r#"
indent = 2
currency_column = 80
thousands = "add"
"#;
        let config: FileConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.indent, Some(2));
        assert_eq!(config.currency_column, Some(80));
        assert_eq!(config.thousands, Some("add".to_string()));
        assert!(config.sort.is_none());
    }

    #[test]
    fn load_sort_bool_true() {
        let config: FileConfig = toml::from_str("sort = true").unwrap();
        assert_eq!(config.sort, Some(SortOrder::Asc));
    }

    #[test]
    fn load_sort_bool_false() {
        let config: FileConfig = toml::from_str("sort = false").unwrap();
        assert_eq!(config.sort, Some(SortOrder::Off));
    }

    #[test]
    fn load_sort_string_asc() {
        let config: FileConfig = toml::from_str(r#"sort = "asc""#).unwrap();
        assert_eq!(config.sort, Some(SortOrder::Asc));
    }

    #[test]
    fn load_sort_string_desc() {
        let config: FileConfig = toml::from_str(r#"sort = "desc""#).unwrap();
        assert_eq!(config.sort, Some(SortOrder::Desc));
    }

    #[test]
    fn load_sort_string_off() {
        let config: FileConfig = toml::from_str(r#"sort = "off""#).unwrap();
        assert_eq!(config.sort, Some(SortOrder::Off));
    }

    #[test]
    fn load_sort_string_true() {
        let config: FileConfig = toml::from_str(r#"sort = "true""#).unwrap();
        assert_eq!(config.sort, Some(SortOrder::Asc));
    }

    #[test]
    fn load_from_string_empty_toml() {
        let config: FileConfig = toml::from_str("").unwrap();
        assert_eq!(config.indent, None);
    }

    #[test]
    fn find_project_config_traverses_upward() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("a").join("b");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(dir.path().join(".beanfmt.toml"), "indent = 2\n").unwrap();
        let config = find_project_config(&sub);
        assert_eq!(config.indent, Some(2));
    }

    #[test]
    fn find_project_config_prefers_dotfile() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("beanfmt.toml"), "indent = 8\n").unwrap();
        std::fs::write(dir.path().join(".beanfmt.toml"), "indent = 2\n").unwrap();
        let config = find_project_config(dir.path());
        assert_eq!(config.indent, Some(2));
    }

    #[test]
    fn find_project_config_falls_back_to_non_dotfile() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("beanfmt.toml"), "indent = 8\n").unwrap();
        let config = find_project_config(dir.path());
        assert_eq!(config.indent, Some(8));
    }

    #[test]
    fn find_project_config_returns_default_when_none() {
        let dir = tempfile::tempdir().unwrap();
        let config = find_project_config(dir.path());
        assert_eq!(config.indent, None);
    }

    #[test]
    fn full_merge_chain() {
        let global = FileConfig {
            indent: Some(2),
            ..FileConfig::default()
        };
        let project = FileConfig {
            currency_column: Some(80),
            ..FileConfig::default()
        };
        let cli = FileConfig {
            sort: Some(SortOrder::Asc),
            ..FileConfig::default()
        };
        let options = global.merge(project).merge(cli).into_options();
        assert_eq!(options.indent, 2);
        assert_eq!(options.currency_column, 80);
        assert_eq!(options.cost_column, 75);
        assert_eq!(options.sort, SortOrder::Asc);
    }

    #[test]
    fn cli_overrides_project_overrides_global() {
        let global = FileConfig {
            indent: Some(2),
            ..FileConfig::default()
        };
        let project = FileConfig {
            indent: Some(4),
            ..FileConfig::default()
        };
        let cli = FileConfig {
            indent: Some(8),
            ..FileConfig::default()
        };
        let options = global.merge(project).merge(cli).into_options();
        assert_eq!(options.indent, 8);
    }
}
