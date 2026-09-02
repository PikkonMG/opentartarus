# OpenTartarus

Linux app for the Razer Tartarus V2 (`1532:022b`) and Tartarus Pro (`1532:0244`). It uses Synapse-like profiles and key remaps. Lighting is a thin OpenRazer passthrough; without OpenRazer, remaps still work and lighting controls stay off.

The remap process is a tray daemon. The UI is iced. Close the window and it hides; remaps keep running until you quit from the tray.

Source: https://github.com/PikkonMG/opentartarus

## What you need

- Linux
- A Tartarus V2 or Tartarus Pro
- Rust, to build from this tree
- OpenRazer, if you want lighting (keys still work without it)

## How to build and run

From this checkout, start the daemon (tray, remaps, lighting):

```
cargo run -p opentartarus-daemon
```

Then open the window:

```
cargo run -p opentartarus-ui
```

If `cargo` on your PATH is a rustup shim and fails with `unknown proxy name: Cursor-3.17.8-x86_64`, call the real binary instead:

```
$(rustup which cargo) run -p opentartarus-daemon
```

The same substitution works for the UI crate and for tests. You can also use the `cargo` under your stable rustup toolchain directory.

## How to test

```
cargo test --workspace
```

The Tartarus V2 plus OpenRazer sysfs check is skipped unless you set:

```
OPENTARTARUS_HW_TEST=1
```

## First-run permissions

The keypad and `/dev/uinput` need group `opentartarus`. The udev rule in `packaging/udev/99-opentartarus.rules` assigns that group on Tartarus V2/Pro nodes and uinput.

`opentartarus-fix-permissions` (polkit helper, policy in `packaging/polkit/com.opentartarus.fix-permissions.policy`) creates the group, adds your user, and writes the udev rule if it is missing. The UI "Fix permissions" button runs that helper via `pkexec` at `/usr/libexec/opentartarus-fix-permissions`.

After a successful fix, unplug the Tartarus, wait a second, and plug it back in. If it still cannot open the device, sign out and sign back in so the new group applies.

Login start files are in `packaging/`: a systemd user unit (`packaging/systemd/opentartarus-daemon.service`) and `packaging/autostart/opentartarus-daemon.desktop`. The window launcher is `packaging/desktop/opentartarus.desktop`.

## Layout

- `crates/`
  - `opentartarus-core`: profile format, remap types, IPC, shipped game layouts in `profiles/`
  - `opentartarus-daemon`: tray process, remaps, lighting
  - `opentartarus-ui`: iced window
  - `opentartarus-fix-permissions`: polkit helper that installs the udev rule
- `packaging/`: udev, systemd user unit, polkit policy, desktop entry, login autostart
