#[derive(serde::Deserialize)]
pub struct Settings {
    pub app: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub db_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}

pub fn get_settings() -> Result<Settings, config::ConfigError> {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());

    let base_path = std::env::current_dir().expect("Could not find current directory");
    let settings_path = base_path.join("settings");

    let settings = config::Config::builder()
        .add_source(config::File::from(settings_path.join("base")))
        .add_source(config::File::from(settings_path.join(app_env)))
        .build()?;

    settings.try_deserialize()
}
