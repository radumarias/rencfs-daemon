# rencfs-daemon

An encrypted file system in Rust that mounts with FUSE on Linux. It can be used to create encrypted directories.

It uses [rencfs](https://github.com/radumarias/rencfs) and can be installed as a systemd service and configure via YAML files.

You can define encrypted directories with their mount points, defined as vaults. It exposes a gRPC server to interract with, you can build your own custom GUI client over it.
