# Ussal Bench

[![dependency status](https://deps.rs/repo/github/rukai/ussal-bench/status.svg)](https://deps.rs/repo/github/rukai/ussal-bench)

Ussal is a microbenchmarking framework that provides an out of the box way to perform benchmarking in the only way I consider reasonable.
This includes:

* ussal-bench a benchmark harness that measures both instruction count and walltime
  * Instruction counting is the most stable metric available when benchmarking but walltime is also important to remain anchored in reality.
* ussal-server a service that will run on a cluster of test machines and run benchmarks for you.
* ussal-client a local client for sending benchmarks compiled from your dev environment to ussal-server
  * No need to stop using your machine while benchmarks are running.
  * Create a large cluster of raspberry pis to run your benchmarks concurrently and complete the run in a fraction of the time it takes locally.
* ussal-client contains CI infrastucture that uses ussal-server to get consistent results.
  * Keep track of performance history by commit, displayed in graphs.
  * Detect performance regressions and improvements in PRs

The above list is currently just a wishlist and entirely unimplemented.

## Platform support

Bencher should run locally on windows, mac and linux.

Server and client only supports linux and currently only supports running benches compiled with musl.
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

### run ussal-client

Run `cargo ussal-client`

## Runners

Runners need access to nsjail.
If your OS doesnt package it, then consider building by following: <https://github.com/google/nsjail/issues/216>

### How does this compare to [bencher.dev](https://bencher.dev)

bencher.dev is a service that will record, compare and analyze benchmark results.
It does not provide any service for actually running your benchmarks, that must occur either within CI machines or locally.

Ussal on the other hand is a client+server for running benchmarks across a cluster of machines.
This requires the project to setup the hardware for such a cluster which is quite an investment.
But does have some advantages (as listed at the top of this readme)

But also bencher.dev is a lot more professional and complete, this is just a small hobby project!
