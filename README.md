# OpenTartarus

Linux profiles and key customization for the Razer Tartarus V2 and Tartarus Pro.

**Source:** https://github.com/PikkonMG/opentartarus

## Install

1. Install the OpenTartarus package from your distro or the release `.deb`.
2. Plug in the Tartarus.
3. After login you should see an OpenTartarus icon in the tray. Left-click it.
4. If the window says it can’t talk to the keypad, click **Fix permissions**, then unplug the Tartarus, wait a second, and plug it back in. If it still fails, sign out and sign back in.
5. Click a game name on the left. Keys apply immediately. Click a key to record a new bind.

Close the window whenever you want — remaps keep working. Quit from the tray icon when you want them to stop.

Lighting uses OpenRazer. If OpenRazer is not installed, keys still work and lighting controls stay off.

## Layout

- `crates/` — application source
  - `opentartarus-core` — profile format, remap types, IPC, shipped game layouts in `profiles/`
  - `opentartarus-daemon` — tray process, remaps, lighting
  - `opentartarus-ui` — iced window
  - `opentartarus-fix-permissions` — polkit helper that installs the udev rule
- `packaging/` — udev, systemd user unit, polkit policy, desktop entry, login autostart
