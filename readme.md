# Rusty benchmark tooling experiments

Make sure to copy over linker settings from `.cargo/config` for cross compilation setup

## Compiling server

Install musl package as `ring` requires `aarch64-linux-musl-gcc`

* Arch - <https://aur.archlinux.org/packages/aarch64-linux-musl>
* Others - TODO

Maybe we should use alpine linux to host since we dont need glibc?

## Compiling ussal-bench

For now compile with musl, but we can setup a glibc chroot to run in the nsjail
