pub struct Config {
    pub dist_base_url: String,
    pub default_version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dist_base_url: String::from("https://nodejs.org/dist"),
            default_version: String::from("latest"),
        }
    }
}
