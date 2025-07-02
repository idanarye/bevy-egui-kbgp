# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.25.0 - 2025-07-02
### Changed
- Upgrade bevy_egui to 0.35.

## 0.24.0 - 2025-04-26
### Changed
- Upgrade Bevy to 0.16 and bevy_egui to 0.34.

## 0.23.0 - 2025-02-17
### Changed
- Update bevy_egui version to 0.33.

## 0.22.0 - 2025-01-09
### Changed
- Update bevy_egui version to 0.32.

## 0.21.0 - 2024-11-30
### Changed
- Update Bevy version to 0.15 and bevy_egui version to 0.31.

## 0.20.0 - 2024-07-06
### Changed
- Update Bevy version to 0.14 and bevy_egui version to 0.28.

## 0.19.0 - 2024-05-08
### Changed
- Update bevy_egui version to 0.27.

## 0.18.0 - 2024-03-19
### Changed
- Update bevy_egui version to 0.26.

## 0.17.0 - 2024-02-22
### Changed
- Update Bevy version to 0.13 and bevy_egui version to 0.25.

## 0.16.0 - 2023-11-06
### Changed
- Update Bevy version to 0.12 and bevy_egui version to 0.23.

## 0.15.0 - 2023-10-19
### Changed
- [**BREAKING**] Update bevy_egui version to 0.22.

  This is a breaking change because starting from this version (more accurately
  version 0.23 of egui) egui will process the arrow keys and use them for
  navigation. This means that unless the default navigation is disabled using
  `KbgpSettings::disable_default_navigation`, using `KbgpNavBindings::default`
  will process the arrow keys twice.

  `KbgpNavBindings::default_gamepad_only` should be used instead.

## 0.14.0 - 2023-04-11
### Changed
- Update Bevy version to 0.11.

## 0.13.0 - 2023-05-28
### Added
- `kbgp_click_released`, `kbgp_user_action_released` and
  `kbgp_activate_released` for activating clicks and user actions after the
  button was released.

## 0.12.0 - 2023-03-08
### Changed
- Update Bevy version to 0.10 and bevy_egui version to 0.20.

## 0.11.0 - 2023-02-01
### Changed
- Update bevy_egui version to 0.19 which updates the `arboard` dependency and fixes
  panics on some devices due to missing swapchain textures.

## 0.10.0 - 2022-12-14
### Changed
- Update bevy_egui version to 0.18 which means egui version gets updated to 0.20.

## 0.9.0 - 2022-11-13
### Changed
- Update Bevy version to 0.9 and bevy_egui version to 0.17.

## 0.8.0 - 2022-10-05
### Changed
- Update bevy_egui version to 0.16 which means egui version gets updated to 0.19.

## 0.7.0 - 2022-08-01
### Changed
- Update Bevy version to 0.8 and bevy_egui version to 0.15.

## 0.6.0 - 2022-05-03
### Changed
- Update bevy_egui version to 0.14 which means egui version gets updated to 0.18.

## 0.5.0 - 2022-04-17
### Changed
- Update Bevy version to 0.7 and bevy_egui version to 0.13.

## 0.4.0 - 2022-04-13
### Fixed
- [**BREAKING**] A typo - "lost of focus" -> "loss of focus". But it's in a
  public identifiers so it's a breaking change.

## 0.3.0 - 2022-04-13
### Added
- [**BREAKING**] Settings to disable egui's default navigation and activation.
- `kbgp_focus_label` and `kbgp_set_focus_label` to control the focus.
- [**BREAKING**] Setting to prevent egui from ever losing focus.
- [**BREAKING**] Setting to give focus to widgets hovered by the mouse.

## 0.2.1 - 2022-02-02
### Fixed
- Changed the navigation code so that KBGP navigation will make egui emit `FocusGained` output
  events.
- Fix diagonal navigation.

## 0.2.0 - 2022-03-23
### Added
- Ability to set navigation binding.
- User actions.

### Changed
- Replace individual navigation navigation methods with a single method + enum.
- Replace `navigate_keyboard_default` with `navigate_keyboard_by_binding`.
- Replace `navigate_gamepad_default` with `navigate_gamepad_by_binding`.

## 0.1.2 - 2022-03-22
### Fixed
- Fix a bug where `kbgp_initial_focus` would only be applied on the first time
  a UI is shown.

## 0.1.1 - 2022-03-21
### Fixed
- Fix a bug where `kbgp_system_default_input` would happen before bevy_egui's `begin_frame`.

## 0.1.0 - 2022-03-21
### Added
- KBGP plugin, systems, and extension methods.
- Navigation with keyboard and gamepads.
- Mechanisms for building key-setting UI, with support for chords.
