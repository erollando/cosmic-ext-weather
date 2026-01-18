use reqwest::header;
use serde::Deserialize;

use crate::config::APP_ID;

#[derive(Clone, Debug)]
pub struct GeocodedPlace {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub admin1: Option<String>,
    pub country: Option<String>,
}

impl GeocodedPlace {
    pub fn label(&self) -> String {
        let mut parts = vec![self.name.as_str()];

        if let Some(admin1) = self.admin1.as_deref()
            && !admin1.is_empty()
        {
            parts.push(admin1);
        }

        if let Some(country) = self.country.as_deref()
            && !country.is_empty()
        {
            parts.push(country);
        }

        parts.join(", ")
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    admin1: Option<String>,
    country: Option<String>,
}

#[derive(Deserialize)]
pub struct WeatherApiResponse {
    properties: Properties,
}

#[derive(Deserialize)]
struct Properties {
    timeseries: Vec<Timeseries>,
}

#[derive(Deserialize)]
struct Timeseries {
    data: Data,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Data {
    instant: Instant,
    next_1_hours: Next1Hours,
    next_6_hours: Next6Hours,
    next_12_hours: Next12Hours,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Instant {
    details: InstantDetails,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct InstantDetails {
    air_pressure_at_sea_level: f64,
    air_temperature: f64,
    cloud_area_fraction: f64,
    relative_humidity: f64,
    wind_from_direction: f64,
    wind_speed: f64,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next1Hours {
    summary: Summary,
    details: Next1HoursDetails,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next6Hours {
    summary: Summary,
    details: Next6HoursDetails,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next12Hours {
    summary: Summary,
    details: Next12HoursDetails,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Summary {
    symbol_code: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next1HoursDetails {
    precipitation_amount: f64,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next6HoursDetails {
    precipitation_amount: f64,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Next12HoursDetails {
    precipitation_amount: f64,
}

pub async fn get_location_forecast(
    latitude: String,
    longitude: String,
) -> Result<i32, reqwest::Error> {
    let url = format!(
        "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={latitude}&lon={longitude}",
    );

    let request_builder = reqwest::Client::new()
        .get(url)
        .header(header::USER_AGENT, APP_ID);

    let response = request_builder.send().await?;
    let data = response.json::<WeatherApiResponse>().await?;

    let current_temperature = data
        .properties
        .timeseries
        .first()
        .map(|d| d.data.instant.details.air_temperature as i32)
        .unwrap_or(0);

    Ok(current_temperature)
}

pub async fn geocode_place(query: String) -> Result<Vec<GeocodedPlace>, reqwest::Error> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Ok(vec![]);
    }

    let response = reqwest::Client::new()
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[
            ("name", query.as_str()),
            ("count", "5"),
            ("language", "en"),
            ("format", "json"),
        ])
        .header(header::USER_AGENT, APP_ID)
        .send()
        .await?;

    let data = response.json::<GeocodingResponse>().await?;
    let results = data
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|result| GeocodedPlace {
            name: result.name,
            latitude: result.latitude,
            longitude: result.longitude,
            admin1: result.admin1,
            country: result.country,
        })
        .collect();

    Ok(results)
}
