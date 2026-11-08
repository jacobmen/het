use std::path::{Path, PathBuf};

pub enum XdgVar {
    Missing,
    Present { value: String },
}

pub fn data_home_dir(var: &XdgVar, home: &str) -> PathBuf {
    let base = match var {
        XdgVar::Missing => format!("{home}/.local/share"),
        XdgVar::Present { value } if value.is_empty() => format!("{home}/.local/share"),
        XdgVar::Present { value } => value.clone(),
    };
    PathBuf::from(base).join("het")
}

pub fn config_home_dir(var: &XdgVar, home: &str) -> PathBuf {
    let base = match var {
        XdgVar::Missing => format!("{home}/.config"),
        XdgVar::Present { value } if value.is_empty() => format!("{home}/.config"),
        XdgVar::Present { value } => value.clone(),
    };
    PathBuf::from(base).join("het")
}

pub fn expand_tilde(path: &Path, home: &str) -> PathBuf {
    match path.strip_prefix("~") {
        Ok(rest) if rest.as_os_str().is_empty() => PathBuf::from(home),
        Ok(rest) => PathBuf::from(home).join(rest),
        Err(_) => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_home_dir_missing() {
        assert_eq!(
            PathBuf::from("/home/user/.local/share/het"),
            data_home_dir(&XdgVar::Missing, "/home/user")
        );
    }

    #[test]
    fn test_data_home_dir_empty_value() {
        assert_eq!(
            PathBuf::from("/home/user/.local/share/het"),
            data_home_dir(
                &XdgVar::Present {
                    value: String::new()
                },
                "/home/user"
            )
        );
    }

    #[test]
    fn test_data_home_dir_present_value() {
        assert_eq!(
            PathBuf::from("/custom/data/het"),
            data_home_dir(
                &XdgVar::Present {
                    value: "/custom/data".to_string(),
                },
                "/home/user"
            )
        );
    }

    #[test]
    fn test_config_home_dir_missing() {
        assert_eq!(
            PathBuf::from("/home/user/.config/het"),
            config_home_dir(&XdgVar::Missing, "/home/user")
        );
    }

    #[test]
    fn test_config_home_dir_empty_value() {
        assert_eq!(
            PathBuf::from("/home/user/.config/het"),
            config_home_dir(
                &XdgVar::Present {
                    value: String::new()
                },
                "/home/user"
            )
        );
    }

    #[test]
    fn test_config_home_dir_present_value() {
        assert_eq!(
            PathBuf::from("/custom/config/het"),
            config_home_dir(
                &XdgVar::Present {
                    value: "/custom/config".to_string(),
                },
                "/home/user"
            )
        );
    }

    #[test]
    fn test_expand_tilde_home_only() {
        assert_eq!(
            PathBuf::from("/home/user"),
            expand_tilde(Path::new("~"), "/home/user")
        );
    }

    #[test]
    fn test_expand_tilde_home_with_subpath() {
        assert_eq!(
            PathBuf::from("/home/user/data/het.db"),
            expand_tilde(Path::new("~/data/het.db"), "/home/user")
        );
    }

    #[test]
    fn test_expand_tilde_trailing_slash() {
        assert_eq!(
            PathBuf::from("/home/user"),
            expand_tilde(Path::new("~/"), "/home/user")
        );
    }

    #[test]
    fn test_expand_tilde_other_user_unchanged() {
        assert_eq!(
            PathBuf::from("~other/db"),
            expand_tilde(Path::new("~other/db"), "/home/user")
        );
    }

    #[test]
    fn test_expand_tilde_absolute_path_unchanged() {
        assert_eq!(
            PathBuf::from("/abs/het.db"),
            expand_tilde(Path::new("/abs/het.db"), "/home/user")
        );
    }

    #[test]
    fn test_expand_tilde_relative_path_unchanged() {
        assert_eq!(
            PathBuf::from("rel/het.db"),
            expand_tilde(Path::new("rel/het.db"), "/home/user")
        );
    }
}
