use std::path::PathBuf;

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
}
