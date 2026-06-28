use cosmic::cosmic_config::{
    self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry,
};

const CONFIG_VERSION: u64 = 3;
const LEGACY_CONFIG_VERSION_V2: u64 = 2;
const LEGACY_CONFIG_VERSION_V1: u64 = 1;

pub const APP_ID: &str = "io.github.cosmic_utils.weather-applet";
pub const SUN_ICON: &str = "io.github.cosmic_utils.weather-applet-symbolic-sun";
pub const MOON_ICON: &str = "io.github.cosmic_utils.weather-applet-symbolic-moon";

#[derive(Clone, Debug, CosmicConfigEntry)]
pub struct WeatherConfig {
    pub latitude: f64,
    pub longitude: f64,
    pub refresh_interval_minutes: u64,
    pub location_name: String,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            refresh_interval_minutes: 10,
            location_name: String::new(),
        }
    }
}

impl WeatherConfig {
    fn config_handler() -> Option<Config> {
        Config::new(APP_ID, CONFIG_VERSION).ok()
    }

    fn migrate_from_v2(config_handler: &Config) -> Option<WeatherConfig> {
        #[derive(Clone, Debug, CosmicConfigEntry)]
        struct V2WeatherConfig {
            pub latitude: f64,
            pub longitude: f64,
            pub refresh_interval_minutes: u64,
        }

        impl Default for V2WeatherConfig {
            fn default() -> Self {
                Self {
                    latitude: 0.0,
                    longitude: 0.0,
                    refresh_interval_minutes: 10,
                }
            }
        }

        let v2_config_handler = Config::new(APP_ID, LEGACY_CONFIG_VERSION_V2).ok()?;
        let v2_config = V2WeatherConfig::get_entry(&v2_config_handler).ok()?;

        let mut migrated = WeatherConfig {
            latitude: v2_config.latitude,
            longitude: v2_config.longitude,
            refresh_interval_minutes: v2_config.refresh_interval_minutes,
            location_name: String::new(),
        };

        if let Err(error) = migrated.set_latitude(config_handler, migrated.latitude) {
            tracing::error!("Error whilst migrating config latitude: {:#?}", error);
        }
        if let Err(error) = migrated.set_longitude(config_handler, migrated.longitude) {
            tracing::error!("Error whilst migrating config longitude: {:#?}", error);
        }
        if let Err(error) =
            migrated.set_refresh_interval_minutes(config_handler, migrated.refresh_interval_minutes)
        {
            tracing::error!(
                "Error whilst migrating config refresh interval: {:#?}",
                error
            );
        }
        if let Err(error) =
            migrated.set_location_name(config_handler, migrated.location_name.clone())
        {
            tracing::error!("Error whilst migrating config location name: {:#?}", error);
        }

        Some(migrated)
    }

    fn migrate_from_v1(config_handler: &Config) -> Option<WeatherConfig> {
        #[derive(Clone, Debug, CosmicConfigEntry)]
        struct V1WeatherConfig {
            pub latitude: f64,
            pub longitude: f64,
            pub use_fahrenheit: bool,
        }

        impl Default for V1WeatherConfig {
            fn default() -> Self {
                Self {
                    latitude: 0.0,
                    longitude: 0.0,
                    use_fahrenheit: false,
                }
            }
        }

        let v1_config_handler = Config::new(APP_ID, LEGACY_CONFIG_VERSION_V1).ok()?;
        let v1_config = V1WeatherConfig::get_entry(&v1_config_handler).unwrap_or_default();

        let mut migrated = WeatherConfig {
            latitude: v1_config.latitude,
            longitude: v1_config.longitude,
            refresh_interval_minutes: WeatherConfig::default().refresh_interval_minutes,
            location_name: String::new(),
        };

        if let Err(error) = migrated.set_latitude(config_handler, migrated.latitude) {
            tracing::error!("Error whilst migrating config latitude: {:#?}", error);
        }
        if let Err(error) = migrated.set_longitude(config_handler, migrated.longitude) {
            tracing::error!("Error whilst migrating config longitude: {:#?}", error);
        }
        if let Err(error) =
            migrated.set_refresh_interval_minutes(config_handler, migrated.refresh_interval_minutes)
        {
            tracing::error!(
                "Error whilst migrating config refresh interval: {:#?}",
                error
            );
        }
        if let Err(error) =
            migrated.set_location_name(config_handler, migrated.location_name.clone())
        {
            tracing::error!("Error whilst migrating config location name: {:#?}", error);
        }

        Some(migrated)
    }

    pub fn config() -> WeatherConfig {
        let Some(config_handler) = Self::config_handler() else {
            return WeatherConfig::default();
        };

        match WeatherConfig::get_entry(&config_handler) {
            Ok(config) => config,
            Err(error) => {
                tracing::error!("Error whilst loading config: {:#?}", error);
                Self::migrate_from_v2(&config_handler)
                    .or_else(|| Self::migrate_from_v1(&config_handler))
                    .unwrap_or_default()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config: WeatherConfig,
    pub config_handler: Option<cosmic_config::Config>,
}

pub fn flags() -> Flags {
    let (config, config_handler) = (WeatherConfig::config(), WeatherConfig::config_handler());

    Flags {
        config,
        config_handler,
    }
}
