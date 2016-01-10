include ./common/common.mk

$(eval $(call ENCAGE_TARGET,arch-bootstrap))
$(eval $(call ENCAGE_DEPENDS,busybox,busybox.mk))

$(BUILD_STAMP).build: $(BUILD_STAMP-busybox)
	ENCAGE_MOUNT="$(BUILD_STAGE):/mnt/build" $(ENCAGE-busybox) run /bin/sh -ec ' \
VERSION=2015.12.01; \
ARCH=i686; \
[ "`uname -m`" = "x86_64" ] && ARCH=x86_64; \
cd "/mnt/build"; \
wget -O - http://mirrors.kernel.org/archlinux/iso/$${VERSION}/archlinux-bootstrap-$${VERSION}-$${ARCH}.tar.gz | tar xz; \
mv ./root.$${ARCH}/* ./; \
rmdir root.$${ARCH}; \
'
	@touch "$@"

$(BUILD_STAMP).pacman-key: $(BUILD_STAMP).build
	$(ENCAGE) run /bin/sh -ec ' \
pacman-key --init; \
pacman-key --populate archlinux; \
'
	@touch "$@"

$(BUILD_STAMP): $(BUILD_STAMP).pacman-key
