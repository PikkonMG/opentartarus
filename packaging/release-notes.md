OpenTartarus is a Linux app for the Razer Tartarus V2, Tartarus Pro and Tartarus V2 Pro. It remaps the keypad with per-game profiles, the way Synapse does on Windows, and it drives the lighting through OpenRazer.

This is a test build. It works with a Tartarus V2 and a Tartarus V2 Pro on Wayland, and a contributor reports it working with a Tartarus Pro. It has not been through many distros. If something does not work, please say so, and say which pad and which distro you have. CONTRIBUTING.md lists the rest of what helps.

## What changed in 0.2.1

### One daemon, one tray icon

Opening the app from the tray could start a second copy of it, and most of the time it did. There were two reasons. A machine can have more than one thing set to start the daemon at login, and when two started in the same moment they both saw a socket with nobody on it, both deleted it and both bound one of their own. Each then had a tray icon, and the one you clicked was not always the one holding the keypad.

The daemon now takes an exclusive lock before it touches the socket at all. The first one to get it does the work. Any other finds it held, says so in the log and stops with a success code, so nothing tries to start it again. Ten daemons started at the same moment leave exactly one running.

Thanks to BerosCerberus for the report, and to colincci for confirming it on a second machine (#4).

### Profiles

Any profile can be renamed now, the shipped ones included: select it in the list and press "Rename". Renaming a shipped profile counts as an edit, so that copy stops following the app, and "Revert to shipped" puts its own name back along with the rest of it.

The tray menu's "Profiles" list used to show only the profiles that ship with the app. It now shows yours as well, under the names you gave them.

### The Tartarus V2 key map

The Tartarus V2 key map was wrong. Fifteen of its twenty keys pointed at the wrong slot, so a profile's bindings landed on keys you did not pick. The thumb pad was read as a joystick hat, which the V2 does not have, so the thumb pad did nothing. The mode button was read as F24, which the pad does not send either.

The map is now recorded from a real V2 (`1532:022b`, hardware revision `0200`) with nothing else holding the pad: all 20 keys, the wheel, the thumb pad and the mode button. The V2 turns out to send the same codes as the Tartarus Pro, so the two pads now share one map.

The thumb pad is four directions, not eight. Push a corner and the pad holds two directions at once, so a profile that binds all four still gets the corners. A binding saved on a diagonal is dropped, because no V2 ever sent one.

If you made a profile on a V2 before this, its keys now land where you meant them to.

Thanks to colincci, who reported it in #5 and checked three pads.

One thing to watch: another remapper that grabs keyboards, such as keyd or input-remapper, takes the V2's key nodes for itself, and OpenTartarus then sees no presses from the pad. Stop the other remapper, or leave the keypad out of its configuration.

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
