use std::time::Duration;

use chrono::{Local, Timelike};

use crate::{
    config::{APP_ID, Flags, MOON_ICON, SUN_ICON, WeatherConfig, flags},
    fl,
    weather::{GeocodedPlace, geocode_place, get_location_forecast},
};

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<Weather>(flags())
}

struct Weather {
    core: cosmic::app::Core,
    popup: Option<cosmic::iced::window::Id>,
    config: WeatherConfig,
    config_handler: Option<cosmic::cosmic_config::Config>,
    temperature: i32,
    location_query: String,
    geocoded_places: Vec<GeocodedPlace>,
    geocode_error: Option<String>,
    latitude: String,
    longitude: String,
    refresh_interval_minutes: u64,
    refresh_interval_minutes_input: String,
}

impl Weather {
    fn update_weather_data(&mut self) -> cosmic::app::Task<Message> {
        cosmic::Task::perform(
            get_location_forecast(
                self.config.latitude.to_string(),
                self.config.longitude.to_string(),
            ),
            |result| match result {
                Ok(temperature) => {
                    cosmic::action::Action::App(Message::UpdateTemperature(temperature))
                }
                Err(error) => {
                    tracing::error!("Failed to get location forecast: {error:?}");
                    cosmic::action::Action::App(Message::UpdateTemperature(0))
                }
            },
        )
    }

    fn format_temperature(&self) -> String {
        format!("{}°C", self.temperature)
    }

    fn refresh_interval_duration(&self) -> Duration {
        Duration::from_secs(self.refresh_interval_minutes.max(1) * 60)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleWindow,
    PopupClosed(cosmic::iced::window::Id),
    UpdateTemperature(i32),
    UpdateLocationQuery(String),
    SearchLocation,
    GeocodeCompleted(Result<Vec<GeocodedPlace>, String>),
    SelectGeocodedPlace(GeocodedPlace),
    UpdateLatitude(String),
    UpdateLongitude(String),
    UpdateRefreshIntervalMinutes(String),
}

impl cosmic::Application for Weather {
    type Flags = Flags;
    type Message = Message;
    type Executor = cosmic::SingleThreadExecutor;

    const APP_ID: &'static str = APP_ID;

    fn init(
        core: cosmic::app::Core,
        flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        let latitude = flags.config.latitude;
        let longitude = flags.config.longitude;
        let refresh_interval_minutes = flags.config.refresh_interval_minutes.max(1);
        let location_name = flags.config.location_name.clone();

        (
            Self {
                core,
                popup: None,
                config: flags.config,
                config_handler: flags.config_handler,
                temperature: 0,
                location_query: location_name,
                geocoded_places: vec![],
                geocode_error: None,
                latitude: latitude.to_string(),
                longitude: longitude.to_string(),
                refresh_interval_minutes,
                refresh_interval_minutes_input: refresh_interval_minutes.to_string(),
            },
            cosmic::task::message(Message::Tick),
        )
    }

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::time::every(self.refresh_interval_duration()).map(|_| Message::Tick)
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::UpdateTemperature(value) => {
                self.temperature = value;
            }
            Message::UpdateLocationQuery(value) => {
                self.location_query = value;
            }
            Message::SearchLocation => {
                self.geocode_error = None;
                self.geocoded_places.clear();

                let query = self.location_query.clone();
                return cosmic::Task::perform(geocode_place(query), |result| {
                    cosmic::action::Action::App(Message::GeocodeCompleted(
                        result.map_err(|error| error.to_string()),
                    ))
                });
            }
            Message::GeocodeCompleted(result) => match result {
                Ok(places) => {
                    self.geocoded_places = places;
                }
                Err(error) => {
                    tracing::error!("Failed to geocode place: {error}");
                    self.geocode_error = Some(fl!("location-search-error"));
                }
            },
            Message::SelectGeocodedPlace(place) => {
                let latitude = place.latitude;
                let longitude = place.longitude;

                self.location_query = place.label();
                self.latitude = latitude.to_string();
                self.longitude = longitude.to_string();
                self.geocode_error = None;
                self.geocoded_places.clear();

                if let Some(handler) = &self.config_handler {
                    if let Err(error) = self.config.set_latitude(handler, latitude) {
                        tracing::error!("{error}");
                    }
                    if let Err(error) = self.config.set_longitude(handler, longitude) {
                        tracing::error!("{error}");
                    }
                    if let Err(error) = self
                        .config
                        .set_location_name(handler, self.location_query.clone())
                    {
                        tracing::error!("{error}");
                    }
                }

                return self.update_weather_data();
            }
            Message::Tick => {
                return self.update_weather_data();
            }
            Message::ToggleWindow => {
                if let Some(id) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(
                        id,
                    );
                } else {
                    let new_id = cosmic::iced::window::Id::unique();
                    self.popup.replace(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    return cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                        popup_settings,
                    );
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::UpdateLatitude(value) => {
                self.latitude = value.to_string();
                self.location_query.clear();
                self.geocode_error = None;
                self.geocoded_places.clear();

                if let Some(handler) = &self.config_handler
                    && let Err(error) = self
                        .config
                        .set_latitude(handler, value.parse::<f64>().unwrap_or_default())
                {
                    tracing::error!("{error}")
                }
                if let Some(handler) = &self.config_handler
                    && let Err(error) = self.config.set_location_name(handler, String::new())
                {
                    tracing::error!("{error}")
                }

                return self.update_weather_data();
            }
            Message::UpdateLongitude(value) => {
                self.longitude = value.to_string();
                self.location_query.clear();
                self.geocode_error = None;
                self.geocoded_places.clear();

                if let Some(handler) = &self.config_handler
                    && let Err(error) = self
                        .config
                        .set_longitude(handler, value.parse::<f64>().unwrap_or_default())
                {
                    tracing::error!("{error}")
                }
                if let Some(handler) = &self.config_handler
                    && let Err(error) = self.config.set_location_name(handler, String::new())
                {
                    tracing::error!("{error}")
                }

                return self.update_weather_data();
            }
            Message::UpdateRefreshIntervalMinutes(value) => {
                self.refresh_interval_minutes_input = value.to_string();

                if let Ok(refresh_interval_minutes) = value.parse::<u64>() {
                    let refresh_interval_minutes = refresh_interval_minutes.max(1);
                    self.refresh_interval_minutes = refresh_interval_minutes;

                    if let Some(handler) = &self.config_handler
                        && let Err(error) = self
                            .config
                            .set_refresh_interval_minutes(handler, refresh_interval_minutes)
                    {
                        tracing::error!("{error}")
                    }
                }
            }
        };

        cosmic::Task::none()
    }

    fn view(&self) -> cosmic::Element<'_, Message> {
        let icon = cosmic::widget::icon::from_name(match Local::now().hour() {
            6..18 => SUN_ICON,
            _ => MOON_ICON,
        })
        .size(self.core.applet.suggested_size(true).0)
        .symbolic(true);
        let temperature = self.core.applet.text(self.format_temperature());
        let (major_padding, minor_padding) = self.core.applet.suggested_padding(true);

        let data = if self.core.applet.is_horizontal() {
            cosmic::Element::from(
                cosmic::iced::widget::row![icon, temperature]
                    .align_y(cosmic::iced::alignment::Vertical::Center)
                    .spacing(4)
                    .padding([0, major_padding]),
            )
        } else {
            cosmic::Element::from(
                cosmic::iced::widget::column![icon, temperature]
                    .align_x(cosmic::iced::alignment::Horizontal::Center)
                    .spacing(4)
                    .padding([minor_padding, 0]),
            )
        };

        let button = cosmic::widget::button::custom(data)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_down(Message::ToggleWindow);

        cosmic::widget::autosize::autosize(button, cosmic::widget::Id::unique()).into()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> cosmic::Element<'_, Message> {
        let location_search_row = cosmic::iced::widget::column![
            cosmic::widget::text(fl!("location")),
            cosmic::iced::widget::row![
                cosmic::widget::text_input(fl!("location-placeholder"), &self.location_query)
                    .on_input(Message::UpdateLocationQuery)
                    .width(cosmic::iced::Length::Fill),
                cosmic::widget::button::standard(fl!("search")).on_press(Message::SearchLocation),
            ]
            .spacing(8)
        ]
        .spacing(4);

        let geocode_error = self
            .geocode_error
            .as_ref()
            .map(|msg| cosmic::Element::from(cosmic::widget::text(msg)))
            .unwrap_or_else(|| cosmic::Element::from(cosmic::widget::Space::new().height(0)));

        let geocode_results = if self.geocoded_places.is_empty() {
            cosmic::Element::from(cosmic::widget::Space::new().height(0))
        } else {
            let items = self.geocoded_places.iter().cloned().fold(
                cosmic::iced::widget::column![cosmic::widget::text(fl!("search-results"))]
                    .spacing(4),
                |column, place| {
                    column.push(
                        cosmic::widget::button::standard(place.label())
                            .on_press(Message::SelectGeocodedPlace(place)),
                    )
                },
            );
            cosmic::Element::from(items.spacing(4))
        };

        let latitude_row = cosmic::iced::widget::column![
            cosmic::widget::text(fl!("latitude")),
            cosmic::widget::text_input(fl!("latitude"), &self.latitude)
                .on_input(Message::UpdateLatitude)
                .width(cosmic::iced::Length::Fill)
        ]
        .spacing(4);
        let longitude_row = cosmic::iced::widget::column![
            cosmic::widget::text(fl!("longitude")),
            cosmic::widget::text_input(fl!("longitude"), &self.longitude)
                .on_input(Message::UpdateLongitude)
                .width(cosmic::iced::Length::Fill)
        ]
        .spacing(4);
        let refresh_interval_row = cosmic::iced::widget::column![
            cosmic::widget::text(fl!("refresh-interval-minutes")),
            cosmic::widget::text_input(
                fl!("refresh-interval-minutes-placeholder"),
                &self.refresh_interval_minutes_input
            )
            .on_input(Message::UpdateRefreshIntervalMinutes)
            .width(cosmic::iced::Length::Fill)
        ]
        .spacing(4);

        let data = cosmic::iced::widget::column![
            cosmic::applet::padded_control(location_search_row),
            cosmic::applet::padded_control(geocode_error),
            cosmic::applet::padded_control(geocode_results),
            cosmic::applet::padded_control(cosmic::widget::divider::horizontal::default()),
            cosmic::applet::padded_control(latitude_row),
            cosmic::applet::padded_control(longitude_row),
            cosmic::applet::padded_control(cosmic::widget::divider::horizontal::default()),
            cosmic::applet::padded_control(refresh_interval_row)
        ]
        .padding([16, 0]);

        self.core
            .applet
            .popup_container(cosmic::widget::container(data))
            .into()
    }
}
