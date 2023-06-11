# Ussal Benchmarking

## Platform support

Bencher should run locally on windows, mac and linux.

Server and client support only supports linux and currently only supports running benches compiled with musl.
In the future we can support glibc via a glibc chroot for nsjail.
I have no immediate plans for mac or linux support in the server/client but architecturally it is designed to support such OS's

## Setup a Raspberry Pi 4 as an ussal server with jobs submitted from x86_64

To compile for `aarch64-unknown-linux-musl` on an x86_64 client, download this:

```shell
wget http://musl.cc/aarch64-linux-musl-cross.tgz
tar -xvf aarch64-linux-musl-cross.tgz
```

Then add the extracted `aarch64-linux-musl-cross/bin` to your PATH.

Some distros package this, e.g.

* Arch - <https://aur.archlinux.org/packages/aarch64-linux-musl>

Setup your project with a `.cargo/config.toml`

```toml
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-musl-gcc"
```

### Setup server

`cargo build --package ussal-server`

Then copy to server and run install

Maybe we should use alpine linux to host since we dont need glibc?

### Setup ussal-bench and ussal-client

### run ussal-clinet

Run `cargo ussal-client`

## Runners

Runners need access to nsjail.
If your OS doesnt package it, then consider building by following: <https://github.com/google/nsjail/issues/216>
