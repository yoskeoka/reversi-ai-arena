SHELL := /bin/bash

.PHONY: rust-fmt rust-clippy rust-test wasm-check verify-rust verify-workflows

rust-fmt:
	./tools/rust-ci.sh fmt

rust-clippy:
	./tools/rust-ci.sh clippy

rust-test:
	./tools/rust-ci.sh test

wasm-check:
	./tools/rust-ci.sh wasm

verify-rust:
	./tools/rust-ci.sh verify

verify-workflows:
	./tools/workflow-lint.sh --mode=ci
