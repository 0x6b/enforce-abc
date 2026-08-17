# enforce-abc

Enforces Latin input mode every time an app is activated:

- **macOS:** ABC
- **Windows:** sends the Muhenkan (NonConvert) key, turning the Japanese IME off without switching keyboard layouts

## Install and Usage

```console
$ cargo install --git https://github.com/0x6b/enforce-abc.git
```

Run it directly:

```console
$ enforce-abc # defaults to `start`
```

On macOS, it can also manage a LaunchAgent:

```console
$ enforce-abc register
$ enforce-abc unregister
```

On Windows, configure startup separately if you want it to run automatically at logon.

Logs go to stderr at `info` level by default; override with `RUST_LOG=debug enforce-abc`. When running as a macOS LaunchAgent, stdout/stderr are written to `/tmp/enforce-abc.out.log` and `/tmp/enforce-abc.err.log`.

## License

MIT. See [LICENSE](LICENSE) for details.
