# encage

Yet another container ecosystem that aims to not be as terrible as Docker.
Hopefully won't fall into the same traps!

![](http://imgs.xkcd.com/comics/standards.png)

## Components

- [ocf](https://github.com/arcnmx/ocf-rs):
  OCI/OCF parsing library.
- [encage-runtime](https://github.com/arcnmx/encage-runtime):
  Linux container runtime and an implementation of the OCF.
- [encage-build](https://github.com/arcnmx/encage/tree/master/build):
  A build system for containers.
- [encage-make](https://github.com/arcnmx/encage/tree/master/make):
  A prototype of `encage-build` implemented via Makefile
- [encage-conf](https://github.com/arcnmx/encage/tree/master/conf):
  A container format specification and parsing library.

## Previous Work

Some of these are good, some bad. They've all inspired `encage` in one way or
another.

- [Docker](https://www.docker.com/)
- [LXC](https://linuxcontainers.org/)
- [runC](https://runc.io/) and [OCI](https://github.com/opencontainers)
- [rkt](https://github.com/coreos/rkt)
- [systemd-nspawn](http://www.freedesktop.org/software/systemd/man/systemd-nspawn.html)
- [Vagga](https://github.com/tailhook/vagga)
