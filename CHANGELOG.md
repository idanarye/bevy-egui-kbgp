# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
