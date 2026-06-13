# enforce-abc

Switches the macOS keyboard input source to **ABC** every time an app is activated.

## Install and Usage

```console
$ cargo install --git https://github.com/0x6b/enforce-abc.git
```

Run it directly, or register it as a LaunchAgent to start automatically:

```console
$ enforce-abc            # run in the foreground (defaults to `start`)
$ enforce-abc register   # install a LaunchAgent at ~/Library/LaunchAgents/enforce-abc.plist
$ enforce-abc unregister # remove it
```

To update, unregister the current LaunchAgent, reinstall, then register again:

```console
$ enforce-abc unregister
$ cargo install --git https://github.com/0x6b/enforce-abc.git
$ enforce-abc register
```

Logs go to stderr at `info` level by default; override with `RUST_LOG=debug enforce-abc`. When running as a LaunchAgent, stdout/stderr are written to `/tmp/enforce-abc.out.log` and `/tmp/enforce-abc.err.log`.

## License

MIT. See [LICENSE](LICENSE) for details.
