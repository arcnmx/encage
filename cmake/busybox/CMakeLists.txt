cmake_minimum_required(VERSION 2.6)
project(busybox)
include(../encage.cmake)

set(BUSYBOX_ARCH, x86_64)
ENCAGE_TARGET(busybox)
ENCAGE_DOWNLOAD(busybox "http://www.busybox.net/downloads/binaries/busybox-${BUSYBOX_ARCH}" "/sbin/busybox")
ENCAGE_COMMAND(busybox busybox sh -ec "
	busybox --list-all | {
		while read path; do
			busybox mkdir -p \"\$ENCAGE_ROOT/\$(busybox dirname \"\$path\")\"
			busybox ln -s \"\$ENCAGE_ROOT/sbin/busybox\" \"\$ENCAGE_ROOT/\$path\"
		done
	}"
)
