ENCAGE := $(dir $(lastword $(MAKEFILE_LIST)))/encage
SHELL := /bin/bash

define ENCAGE_TARGET
TARGET 		:= $(1)
BUILD_DIR	:= build/$$(TARGET)
BUILD_STAMP	:= $$(BUILD_DIR)/stamp
BUILD_STAGE	:= $$(BUILD_DIR)/image

export ENCAGE_ROOT := $$(BUILD_STAGE)

$$(shell mkdir -p "$$(ENCAGE_ROOT)")
$$(shell mkdir -p "build/$$(TARGET)/image")

all: $$(BUILD_STAMP)

run: $$(BUILD_STAMP)
	exec $$(ENCAGE) run

clean:
	rm -rf "build/"

.PHONY: all run clean

$$(BUILD_STAMP):
	@touch "$$@"

endef

define ENCAGE_DEPENDS
TARGET-$(1)			:= $(1)
BUILD_DIR-$(1)		:= build/$$(TARGET-$(1))
BUILD_STAMP-$(1)	:= $$(BUILD_DIR-$(1))/stamp
BUILD_STAGE-$(1)	:= $$(BUILD_DIR-$(1))/image
ENCAGE-$(1)			:= env ENCAGE_ROOT=$$(BUILD_STAGE-$(1)) $(ENCAGE)

$$(BUILD_STAMP-$(1)):
	@+$(MAKE) -f "$(2)"
	@touch "$$@"
endef
