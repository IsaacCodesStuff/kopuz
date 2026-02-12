# Rusic

Rusic is a modern, lightweight music player application built with Rust and the Dioxus framework. It provides a clean and responsive interface for managing and enjoying your local music collection.

![Rusic Interface](https://github.com/user-attachments/assets/d5ece9e3-ca7a-4986-929f-c01198ab6ba1)

## Overview

Rusic allows you to scan your local directories for audio files, or you jellyfin library, automatically organizing them into a browsable library. You can navigate by artists, albums, or explore your custom playlists. The application is built for performance and desktop integration, utilizing the power of Rust.

## Features

- **Theming**: Includes dynamic theming support to customize the visual appearance.
- **Native Integration**: Integrates with system media controls (MPRIS) and "Now Playing" displays.
- **Discord RPC**: Embedded RPC included!!!
- **Double Option**: Yes you can also use your jellyfin server to listen to your music coming from your server!

## Installation

### Flatpak (Recommended)

Rusic is available on Flathub (coming soon). To install from source manifest:

```bash
git clone https://github.com/temidaradev/rusic
cd rusic
flatpak-builder --user --install --force-clean build-dir com.temidaradev.rusic.json
flatpak run com.temidaradev.rusic
```

or click on the file and open it with app provider like kde discover

### Build from Source


If you want to hack on it:

```bash
npm install

dx serve --package rusic
```

## Tech Stack

- **Dioxus**: UI Framework
- **Rodio**: Audio playback
- **Lofty**: Metadata parsing
- **TailwindCSS**: All the styling

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=temidaradev/rusic&type=date&legend=top-left)](https://www.star-history.com/#temidaradev/rusic&type=date&legend=top-left)
