OpenTartarus is a Linux app for the Razer Tartarus V2, Tartarus Pro and Tartarus V2 Pro. It remaps the keypad with per-game profiles, the way Synapse does on Windows, and it drives the lighting through OpenRazer.

This is a test build. It works with a Tartarus V2 and a Tartarus V2 Pro on Wayland, and a contributor reports it working with a Tartarus Pro. It has not been through many distros. If something does not work, please say so, and say which pad and which distro you have. CONTRIBUTING.md lists the rest of what helps.

## What changed in 0.2.3

The Tartarus V2 keeps its remaps in driver mode and in normal device mode, so it no longer depends on OpenRazer to hold the pad in driver mode. Remaps also work on a V2 with no OpenRazer installed.

A key you are holding when the profile changes comes back up as the key it went down as, and unplugging the pad lets go of every key.

The daemon starts on the profile you left it on, including one you made yourself. It also shuts down cleanly when your session's service manager stops it.

A profile file the app cannot read is skipped, and the other profiles still show in the list. The file stays where it is.

Thanks to colincci for the report and the diagnosis (#9).

## What it does

- 13 profiles, each built from that game's real default key bindings
- on the Tartarus V2 Pro, a layout of each profile made for its keys and thumbstick
- any key can send a key, a combo, a bare Shift or Ctrl, a mouse button, or a macro
- a key can jump to another profile, or step to the next one
- lighting through OpenRazer, falling back to OpenRGB
- a tray icon, so the remaps keep running with the window closed

## Install

Debian, Ubuntu, Mint, Pop!_OS:

    sudo apt install ./opentartarus_*_amd64.deb

Fedora, openSUSE, RHEL:

    sudo dnf install ./opentartarus-*.x86_64.rpm

Arch, CachyOS, Manjaro, EndeavourOS: build the package from `packaging/arch` with `makepkg -si`. The README has the steps.

Then open the app, open the menu, and press "Fix permissions" once. That adds you to the `opentartarus` group, which is what lets the app open the keypad at all. Unplug the pad and plug it back in. If the app still cannot find it, sign out and back in.

Lighting needs the OpenRazer daemon running. The remaps do not.

## One warning

Every shipped profile sends one action for one press, which is what game publishers allow. Macros and hold-to-repeat are not, in most games, and several ban them outright. The window turns that warning red the moment you switch one on. The Game rules section of the README has the detail.
