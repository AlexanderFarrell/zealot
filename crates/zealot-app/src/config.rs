#[derive(Debug, Clone)]
pub struct ZealotConfig {
    pub database: String,
    pub db_username: Option<String>,
    pub db_password: Option<String>,
    pub db_host: Option<String>,
    pub db_database: Option<String>,
    pub db_filename: String,

    pub media_source: String,
    pub media_path: String,

    pub port: i32,
}

impl ZealotConfig {
    pub fn load_from_env() -> Self {
        Self {
            database: get_env("DATABASE", "postgres"),
            db_username: get_env_optional("DB_USERNAME"),
            db_password: get_env_optional("DB_PASSWORD"),
            db_host: get_env_optional("DB_HOST"),
            db_database: get_env_optional("DB_DATABASE"),
            db_filename: get_env("DB_FILENAME", "./zealot.db"),

            media_source: get_env("MEDIA_SOURCE", "FILESYSTEM"),
            media_path: get_env("MEDIA_PATH", "./public"),

            port: get_env_int("PORT", 8456),
        }
    }

    pub fn get_host_port(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

pub fn get_env_int(var: &str, def: i32) -> i32 {
    let string = get_env(var, "");
    if string == "" {
        return def;
    }
    match string.parse::<i32>() {
        Ok(v) => v,
        Err(_) => def,
    }
}

pub fn get_env(var: &str, def: &str) -> String {
    match std::env::var(var) {
        Ok(v) => {
            if v == "" {
                String::from(def)
            } else {
                v
            }
        }
        Err(err) => String::from(def),
    }
}

pub fn get_env_optional(var: &str) -> Option<String> {
    let value = get_env(var, "");
    if value == "" {
        return Some(value);
    } else {
        return None;
    }
}
