[![Build Status](https://github.com/idanarye/bevy-egui-kbgp/workflows/CI/badge.svg)](https://github.com/idanarye/bevy-egui-kbgp/actions)
[![Latest Version](https://img.shields.io/crates/v/bevy-egui-kbgp.svg)](https://crates.io/crates/bevy-egui-kbgp)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://idanarye.github.io/bevy-egui-kbgp/)

# Bevy egui KBGP - improved keyboard and gamepad usage for egui in Bevy

[egui](https://github.com/emilk/egui) is an immediate mode GUI framework, that can be used inside the [Bevy game engine](https://bevyengine.org/) with [the bevy_egui crate](https://github.com/mvlabat/bevy_egui).

egui is very mouse-oriented, and while it does support tab-navigation, game menus should be
navigatable by the arrow keys (and/or by WASD) and by gamepads.

This is where the bevy-egui-kbgp crate comes in. It allows to navigate egui widgets using the
keyboard's arrow keys and using a gamepad's d-pad and left joystick (by default - all the controls
can be redfined). It also allows activating these buttons from the gamepad.

Try it out in https://idanarye.github.io/bevy-egui-kbgp/demos/example

## Features

* Navigate the GUI using arrow keys and gamepads.
* Activate buttons from gamepads (egui already supports activation from keyboard with Space/Enter)
* Customize all these controls.
* Key assignment.

## Planned features

* Support for comboboxes.
* Figure out how to support navigating out of textboxes.

## Versions

| bevy | egui | bevy_egui | bevy-egui-kbgp |
|------|------|------------|----------------|
| 0.6  | 0.16 | 0.11.1     | 0.1            |

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
