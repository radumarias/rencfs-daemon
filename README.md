# rencfs-daemon

[rencfs](https://github.com/radumarias/rencfs) daemon.

> [!WARNING]
> **This is very early in development. Please do not use it with sensitive data just yet. Please wait for a
stable release.
> It's mostly ideal for experimental and learning projects.**

An encrypted file system in Rust that is mounted with FUSE on Linux. It can be used to create encrypted directories.

It uses [rencfs](https://github.com/radumarias/rencfs) and can be installed as a systemd service and configured via YAML files.

You can define encrypted directories with their mount points, defined as vaults. It exposes a gRPC server to interract with, you can build your own custom GUI client over it.
