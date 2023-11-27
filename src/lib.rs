use std::str::FromStr;

struct VarConfig<T: FromStr + Copy> {
    name: String,
    default_value: Option<T>,
}

impl<T: FromStr + Copy> VarConfig<T> {
    fn try_load(&self) -> Result<T, String>
    where
        <T as FromStr>::Err: std::fmt::Display,
    {
        match std::env::var(&self.name) {
            Result::Ok(s) => match s.parse() {
                Result::Ok(v) => Result::Ok(v),
                Result::Err(e) => match self.default_value {
                    Option::Some(v) => Result::Ok(v),
                    Option::None => Result::Err(format!("Failed to parse '{}': {}", s, e)),
                },
            },
            Result::Err(_) => match self.default_value {
                Option::Some(v) => Result::Ok(v),
                Option::None => Result::Err(format!("Couldn't find variable '{}'", self.name)),
            },
        }
    }
}

macro_rules! config_var {
    ($field:ident, $type:ty, $_field:ident, $fieldDef:expr) => {
        if let Ok(s) = std::env::var(&$_field) {
            if let Ok(v) = s.parse() {
                v
            } else {
                $fieldDef
            }
        } else {
            $fieldDef
        }
    };
    ($field:ident, $type:ty, $_field:ident) => {
        if let Ok(s) = std::env::var(&$_field) {
            match s.parse() {
                Result::Ok(v) => v,
                Result::Err(e) => {
                    return Result::Err(format!("Failed to parse {} as '$type': {}", $_field, e))
                }
            }
        } else {
            return Result::Err(format!(
                "Couldn't find variable '{}' in the environment",
                $_field
            ));
        }
    };
}

#[macro_export]
macro_rules! config {
    (
        $(#[$outer:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$outer_field:meta])*
                $vis_ident:vis $field:ident: $type:ty $( = $fieldDef:expr )?,
            )*
        }

    ) => {

        $(#[$outer])*
        $vis struct $struct_name {
            $(
                $(#[$outer_field])*
                $vis_ident $field: $type,
            )*
        }

        impl $struct_name {
            pub fn try_load() -> Result<Self, String> {
                let env = Self {
                    $(
                        $field: {
                            let _field = stringify!($field)
                                .chars()
                                .map(|x| char::to_ascii_uppercase(&x))
                                .collect::<String>();
                            config_var!($field, $type, _field $(, $fieldDef)?)
                        },
                    )*
                };
                Result::Ok(env)
            }
        }

    };
}

#[cfg(test)]
mod tests {

    struct EnvTemp {
        flag: &'static str,
        original_content: Option<String>,
    }

    impl EnvTemp {
        fn set_var(flag: &'static str, val: &'static str) -> EnvTemp {
            let env = EnvTemp {
                flag,
                original_content: std::env::var(flag).ok(),
            };
            std::env::set_var(flag, val);
            env
        }
    }

    impl Drop for EnvTemp {
        fn drop(&mut self) {
            // reset_var
            if let Some(og) = &self.original_content {
                std::env::set_var(self.flag, og);
            } else {
                std::env::remove_var(self.flag);
            }
        }
    }

    #[test]
    fn test_with_default() {
        let temp_env = [EnvTemp::set_var("ANOTHER_VAR", "abc")];
        config! {
            struct Config {
                pub a_var: String = "123".to_string(),
                pub another_var: String = "def".to_string(),
                pub integer_var: u8 = 123,
            }
        }
        let r_config = Config::try_load();
        assert!(r_config.is_ok());
        let config = r_config.unwrap();
        assert_eq!(config.a_var, "123");
        assert_eq!(config.another_var, "abc");
        assert_eq!(config.integer_var, 123);
        drop(temp_env);
    }

    #[test]
    fn test_no_default() {
        let temp_env = [EnvTemp::set_var("A_VAR", "123")];
        config! {
            struct Config {
                pub a_var: String,
            }
        }
        let r_config = Config::try_load();
        assert!(r_config.is_ok());
        let config = r_config.unwrap();
        assert_eq!(config.a_var, "123");
        drop(temp_env);
    }

    #[test]
    fn test_var_not_found() {
        config! {
            struct Config {
                pub a_var: String,
            }
        }
        let r_config = Config::try_load();
        assert!(r_config.is_err());
    }
}
