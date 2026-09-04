Three ways to install. Pick the one for your distro.

**Debian, Ubuntu, Mint, Pop!_OS**

    sudo apt install ./opentartarus_*_amd64.deb

**Fedora, openSUSE, RHEL**

    sudo dnf install ./opentartarus-*.x86_64.rpm

**Any distro, AppImage**

    chmod +x OpenTartarus-*-x86_64.AppImage
    ./OpenTartarus-*-x86_64.AppImage

After a deb or rpm install, open the app and press "Fix permissions" once. It adds you to the `opentartarus` group. Then unplug the keypad and plug it back in. If the app still cannot see the pad, sign out and back in.

The AppImage cannot install system files, so do that step by hand:

    sudo groupadd -f opentartarus
    sudo usermod -aG opentartarus "$USER"
    sudo curl -fsSL -o /etc/udev/rules.d/99-opentartarus.rules \
      https://raw.githubusercontent.com/PikkonMG/opentartarus/main/packaging/udev/99-opentartarus.rules
    sudo udevadm control --reload-rules

Then sign out and back in, and replug the keypad.

Lighting needs OpenRazer. Remaps work without it.
