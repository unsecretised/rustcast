# Rustcast - an alternative to Raycast thats open source and in rust

> Rustcast, Raycast and PowerToys are productivity tools. They provide a popup
> search bar that people can use to do many things, like opening apps,
> calculators, quick-notes, etc.

![RustCast Demo PreRelease V1](./docs/rustcast-v0-2-91.png)

## Installation:

### Via Homebrew (Recommended):

> Requires Homebrew

```
brew tap unsecretised/tap
brew install --cask rustcast
```

Then launch it via launchpad or spotlight (hopefully the last time you have to
use spotlight :) Have fun!)

### Via github releases

1. Go to the
   [releases page](https://github.com/unsecretised/rustcast/releases/latest)
2. Download the `.zip` file
3. Open it, and if there is a "this app is damaged", run the command
   `xattr -cr <RustCast.app directory here>`

### Build it youself:

1. Clone the repo with `git clone https://github.com/unsecretised/rustcast.git`
1. Install `cargo bundle` with `cargo install cargo-bundle` (Requires cargo to
   be installed)
1. Run `cargo bundle --release` to build RustCast for your system (The App Dir
   will be available at: `target/release/bundle/osx/RustCast.app`)
1. Move the `RustCast.app` directory to your `/Applications/` folder, and launch
   RustCast

## Config:

> I have a high chance of gorfetting (forgetting) to update the config docs, but
> even if I do, you will most likely be able to find my config easily
> [here](https://github.com/unsecretised/rustcast-dotfiles)

The config file should be located at: `~/.config/rustcast/config.toml` RustCast
doesn't create the default configuration for you, but it does use its
[default options](docs/default.toml) Here's a full list of what all you can
configure [The list](docs/config.toml) The blurring and background is still a
bit wonky, and will be fixed in the upcoming releases

## Feature list:

### Planned:

- [ ] Select the options using arrow keys
- [ ] Popup note-taking
- [ ] Plugin Support (Partially implemented on 15/12/2025)
- [ ] Hyperkey - Map CMD + OPT + CTRL + SHIFT to a physical key
- [ ] Ability to pick between tabs in firefox / chromium browsers - using
      [Puppeteer](https://pptr.dev/)
- [ ] Cross platform support

### Finished:

- [x] Autoload installed apps 11/11/2025
- [x] Search through apps 11/11/2025
- [x] Generate [randomvar](https://github.com/Nazeofel) (between 0 and 100) via
      the app. Simply type `randomvar` and it will generate the num for you
- [x] Image icons next to the text 13/12/2025
- [x] Scrollable options 12/12/2025
- [x] Customisable themes (13/12/2025)
  - [x] Configurable colours
- [x] Spotify control - Ability to control spotify via the app
- [x] Allow variables to be passed into custom shell scripts.
- [x] Google your query. Simply type your query, and then put a `?` at the end,
      and press enter
- [x] Calculator (27/12/2025)
- [x] Clipboard History (29/12/2025) This works by typing `cbhist` to enter the
      cliboard history page, which allows u to access your clipboard history,
- [x] Blur / transparent background (7/1/2026)
      and then use `main` to switch back, or just open an close the app again
- [x] Tray icons (8/1/2026)

## If you like rustcast, consider starring it on github :)

[![Star History Chart](https://api.star-history.com/svg?repos=unsecretised/rustcast&type=date&legend=top-left)](https://www.star-history.com/#unsecretised/rustcast&type=date&legend=top-left)

## Motivations:

I didn't want to pay for raycast + wanted to get better at rust. Raycast in
itself is one of the most useful productivity apps in my opinion, and it is
truly an underappreiciated marvel of computer engineering
