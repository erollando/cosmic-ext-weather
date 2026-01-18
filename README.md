# Simple weather info applet for cosmic

This repository is a fork of `cosmic-utils/cosmic-ext-applet-weather` (original author: rwxroot). Maintained here by `e.rollando@duck.com`.

<p align="center">
    <img alt="Applet Screenshot" src="https://github.com/cosmic-utils/cosmic-ext-applet-weather/blob/main/data/applet_screenshot_1.png">
</p>

<p align="center">
    <img alt="Applet Screenshot" src="https://github.com/cosmic-utils/cosmic-ext-applet-weather/blob/main/data/applet_screenshot_2.png">
</p>

<p align="center">
    <img alt="Applet Screenshot" src="https://github.com/cosmic-utils/cosmic-ext-applet-weather/blob/main/data/applet_screenshot_3.png">
</p>

<p align="center">
    <img alt="Applet Screenshot" src="https://github.com/cosmic-utils/cosmic-ext-applet-weather/blob/main/data/applet_screenshot_4.png">
</p>

## Installation

### Flatpak

Depending on how you've installed COSMIC Desktop, the Weather applet may show up in your app store by default. In COSMIC Store it should be under the "COSMIC Applets" category.

If the applet does not show up in your app store, you'll need to add `cosmic-flatpak` as a source:

```sh
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
```

Then, proceed to your preferred app store and search for Weather applet.

### Manual

The applet can be installed using the following steps:

```sh
sudo apt install libxkbcommon-dev just
git clone https://github.com/cosmic-utils/cosmic-ext-applet-weather.git
cd cosmic-ext-applet-weather
just build
just install
```

`libxkbcommon-dev` is required by `smithay-client-toolkit`

This installs into `~/.local` (`~/.local/bin` and `~/.local/share/...`). Ensure `~/.local/bin` is on your `PATH`.

#### Manual (system install)

```sh
sudo apt install libxkbcommon-dev just
git clone https://github.com/cosmic-utils/cosmic-ext-applet-weather.git
cd cosmic-ext-applet-weather
just build
sudo just install-system
```

## Configuration

The applet provides a graphical interface for searching a location name (which fills in coordinates) and setting the refresh interval. For manual configuration, follow the steps below.

Note: weather is fetched using `latitude`/`longitude`. `location_name` is optional and only affects the UI field.

```sh
cd ~/.config/cosmic/io.github.cosmic_utils.weather-applet/v3/
```

Add latitude:

```
touch latitude
echo "12.123163" > latitude
```

Add longitude:

```
touch longitude
echo "23.811234" > longitude
```

Set refresh interval (minutes):

```
touch refresh_interval_minutes
echo "10" > refresh_interval_minutes
```

Set location name (optional; used for the UI field):

```
touch location_name
echo "Helsinki" > location_name
```

To refresh the applet simply run `pkill cosmic-panel`

## Uninstall

To uninstall a user install (`~/.local`), run:

```sh
just uninstall
```

To uninstall a system install (`/usr`), run:

```sh
sudo just uninstall-system
```
