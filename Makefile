# Native target dir so a FUSE/noexec checkout still links.
TARGET ?= /tmp/ai-foh-target
export CARGO_TARGET_DIR := $(TARGET)

.PHONY: test demo dsp fmt ci

test:
	cargo test --workspace

demo:
	cargo run -p foh-engine -- --map maps/demo.map

dsp:
	cmake -S cpp/foh_dsp -B /tmp/foh_dsp_build
	cmake --build /tmp/foh_dsp_build
	/tmp/foh_dsp_build/foh_dsp_selftest

fmt:
	cargo fmt --all -- --check

ci: test demo dsp
