# Installing `ipod.service`

A systemd unit that runs `ipod serve /dev/iap0` in the background and bridges
the iPod accessory protocol to BlueZ's `org.bluez.MediaPlayer1` interface and
BlueALSA digital audio playback.

The unit file is deliberately *not* installed automatically &mdash; these
instructions walk through placing the files yourself.

## Prerequisites

- Linux with `systemd`.
- `bluez` 5.x running and providing the system-bus service `org.bluez`.
- BlueALSA installed, with `bluealsa.service` available and `bluealsa-aplay`
  on the service user's `PATH`.
- The [`ipod-gadget`](https://github.com/oandrew/ipod-gadget) kernel module
  loaded and creating `/dev/iap0`.
- A compiled `ipod` binary (see the top-level [README](../../README.md)).

## 1. Install the binary

Cross-compile for your target if necessary, then place the binary somewhere
on the target's filesystem. The unit file assumes `/usr/local/bin/ipod`:

```sh
# on the build host
GO111MODULE=on GOOS=linux GOARCH=arm GOARM=6 \
    go build -o ipod github.com/oandrew/ipod/cmd/ipod

# on the target
sudo install -m 0755 ipod /usr/local/bin/ipod
```

If you install it elsewhere, edit `ExecStart=` in `ipod.service` accordingly.

`ipod serve` starts `bluealsa-aplay` by default when the iPod link enters
digital-audio initialization. The `serve` process fails if `bluealsa-aplay` is
missing, cannot be started, or exits while `serve` is running. If your
distribution installs the executable outside the service user's `PATH`, add the
override to `ExecStart=`:

```ini
ExecStart=/usr/local/bin/ipod serve --bluealsa-aplay /usr/bin/bluealsa-aplay /dev/iap0
```

Additional `bluealsa-aplay` options can be passed with repeated
`--bluealsa-aplay-arg` flags:

```ini
ExecStart=/usr/local/bin/ipod serve --bluealsa-aplay-arg=--pcm=default /dev/iap0
```

## 2. Install the unit file

```sh
sudo install -m 0644 contrib/systemd/ipod.service /etc/systemd/system/ipod.service
sudo systemctl daemon-reload
```

## 3. Enable and start

Enable the service so it starts on boot and whenever `/dev/iap0` appears:

```sh
sudo systemctl enable --now ipod.service
```

Check status and follow logs:

```sh
systemctl status ipod.service
journalctl -u ipod.service -f
```

## 4. Verify the BlueZ bridge

With a phone (or other A2DP/AVRCP source) connected over Bluetooth and
actively playing audio, BlueZ exposes a `MediaPlayer1` object. You can
confirm the bridge is seeing it by checking the service log for a line like:

```
bluez: using MediaPlayer1 at /org/bluez/hci0/dev_XX_XX_XX_XX_XX_XX/player0
```

Track metadata (title / artist / album) and play status should now be
propagated to the head unit over the iAP link.

## 5. Verify BlueALSA playback

The shipped unit wants and orders itself after `bluealsa.service`. During
digital-audio initialization, `ipod serve` starts `bluealsa-aplay` and keeps it
under the `serve` process. Check the logs if startup fails or playback stops:

```sh
journalctl -u ipod.service -u bluealsa.service -f
```

If the log shows that `bluealsa-aplay` is missing or exited, install the
BlueALSA player package, fix the executable path with `--bluealsa-aplay`, or
adjust its arguments with repeated `--bluealsa-aplay-arg` flags.

## Running as an unprivileged user (optional)

The shipped unit runs as `root` because that is the path of least resistance
for accessing `/dev/iap0` *and* BlueZ over the system bus. To run as an
unprivileged user, for example `ipod`:

1. Create the user:

   ```sh
   sudo useradd --system --no-create-home --shell /usr/sbin/nologin ipod
   sudo usermod -aG bluetooth ipod
   ```

2. Make `/dev/iap0` accessible. Drop a udev rule at
   `/etc/udev/rules.d/70-ipod-gadget.rules`:

   ```udev
   KERNEL=="iap0", MODE="0660", GROUP="ipod"
   ```

   Then reload udev:

   ```sh
   sudo udevadm control --reload
   sudo udevadm trigger
   ```

3. Allow the `ipod` user to use the relevant BlueZ interfaces. Create
   `/etc/dbus-1/system.d/ipod.conf`:

   ```xml
   <!DOCTYPE busconfig PUBLIC
     "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
     "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
   <busconfig>
     <policy user="ipod">
       <allow send_destination="org.bluez"/>
       <allow send_interface="org.bluez.MediaPlayer1"/>
       <allow send_interface="org.freedesktop.DBus.Properties"/>
       <allow send_interface="org.freedesktop.DBus.ObjectManager"/>
     </policy>
   </busconfig>
   ```

   Reload the bus:

   ```sh
   sudo systemctl reload dbus
   ```

4. Edit `/etc/systemd/system/ipod.service` and change:

   ```ini
   User=ipod
   Group=ipod
   ```

5. Reload and restart:

   ```sh
   sudo systemctl daemon-reload
   sudo systemctl restart ipod.service
   ```

## Uninstall

```sh
sudo systemctl disable --now ipod.service
sudo rm /etc/systemd/system/ipod.service
sudo systemctl daemon-reload
sudo rm /usr/local/bin/ipod
```

## Common tweaks

- **Verbose logging** &mdash; append `-d` to the command:
  `ExecStart=/usr/local/bin/ipod -d serve /dev/iap0`.
- **Write a trace to disk** for debugging:
  `ExecStart=/usr/local/bin/ipod serve -w /var/log/ipod.trace /dev/iap0`
  (make sure the parent directory is writable by the service user and
  relax `ProtectSystem=` if needed).
- **Different device node** &mdash; change both the `BindsTo=`/`After=`
  `.device` units and the `ExecStart=` arguments. For `/dev/iap1` those
  become `dev-iap1.device` and `serve /dev/iap1`.
- **Different `bluealsa-aplay` executable** &mdash; append
  `--bluealsa-aplay /path/to/bluealsa-aplay` before the device argument.
- **Extra `bluealsa-aplay` arguments** &mdash; append one
  `--bluealsa-aplay-arg VALUE` per argument before the device argument.
