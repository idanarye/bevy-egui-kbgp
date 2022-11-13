# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.9.0 - 2022-11-13
- Update Bevy version to 0.9 and bevy-egui version to 0.17.

## 0.8.0 - 2022-10-05
- Update bevy-egui version to 0.16 which means egui version gets updated to 0.19.

## 0.7.0 - 2022-08-01
- Update Bevy version to 0.8 and bevy-egui version to 0.15.

## 0.6.0 - 2022-05-03
- Update bevy-egui version to 0.14 which means egui version gets updated to 0.18.

## 0.5.0 - 2022-04-17
- Update Bevy version to 0.7 and bevy-egui version to 0.13.

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
