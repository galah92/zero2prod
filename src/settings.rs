use config::{Config, ConfigError, Environment, File};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub email_base_url: String,
    pub email_auth_token: String,
    pub email_sender: String,
}

pub fn get_settings() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Could not find current directory");
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
    let settings_path = base_path.join("settings").join(app_env);

    let settings = Config::builder()
        .add_source(Environment::default().try_parsing(true))
        .add_source(File::from(settings_path))
        .build()?;

    settings.try_deserialize()
}
