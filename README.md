RencFs daemon

<a href="https://www.buymeacoffee.com/xorio42"><img src="https://img.buymeacoffee.com/button-api/?text=Buy me a coffee&emoji=☕&slug=xorio42&button_colour=FFDD00&font_colour=000000&font_family=Cookie&outline_colour=000000&coffee_colour=ffffff" /></a>

> [!WARNING]
> **This is very early in development. Please do not use it with sensitive data just yet. Please wait for a
stable release.
> It's mostly ideal for experimental and learning projects.**

An encrypted file system in Rust that mounts with FUSE on Linux. It can be used to create encrypted directories.

It uses [rencfs](https://github.com/radumarias/rencfs) and can be installed as a systemd service and configured via YAML files.

You can define encrypted directories with their mount points, defined as vaults. It exposes a gRPC server to interract with, you can build your own custom GUI client over it.
