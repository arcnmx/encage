include ./common/common.mk

$(eval $(call ENCAGE_TARGET,busybox))

$(BUILD_DIR)/busybox-x86_64:
	curl -L http://www.busybox.net/downloads/binaries/busybox-x86_64 -o "$@"

$(BUILD_STAGE)/sbin/busybox: $(BUILD_DIR)/busybox-x86_64
	@mkdir -p "$(BUILD_STAGE)/"{bin,sbin,usr/bin,usr/sbin}
	cp "$<" "$@"
	@chmod 6755 "$@"

$(BUILD_STAMP).links: $(BUILD_STAGE)/sbin/busybox
	$(ENCAGE) run busybox sh -ec 'busybox --list-all | busybox xargs -n1 -I PATH busybox ln -sf "$$ENCAGE_ROOT/sbin/busybox" "$$ENCAGE_ROOT/PATH"'
	@touch "$@"

$(BUILD_STAMP): $(BUILD_STAMP).links
