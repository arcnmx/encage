# encage-make

A simplistic Makefile build system for Linux containers.

## Examples

### busybox

    $ sudo make -f busybox.mk run

Builds and throws you into a busybox shell. Used as an example of an
image to be used to build other images.

### arch-bootstrap

    $ sudo make -f arch-bootstrap.mk

Creates an environment that can be used to bootstrap an Arch Linux based image.

## Dependencies

- `unshare` from util-linux
- `curl` for downloading the busybox binary
