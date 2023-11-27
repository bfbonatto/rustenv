#[macro_export]
macro_rules! config {
    (
        $(#[$outer:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$outer_field:meta])*
                $vis_ident:vis $field:ident: $type:ty = $fieldDef:expr,
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

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $fieldDef,
                    )*
                }
            }
        }

        impl $struct_name {
            pub fn load() -> Self {
                let mut env = Self::default();
                $(
                    let _field = stringify!($field)
                        .chars()
                        .map(|x| char::to_ascii_uppercase(&x))
                        .collect::<String>();
                    if let Ok(s) = std::env::var(&_field) {
                        if let Ok(v) = s.parse() {
                            env.$field = v;
                        }
                    }
                )*
                env
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
        let config = Config::load();
        assert_eq!(config.a_var, "123");
        assert_eq!(config.another_var, "abc");
        drop(temp_env);
    }
}
