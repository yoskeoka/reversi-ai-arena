SHELL := /bin/bash

.PHONY: rust-fmt rust-clippy rust-test wasm-check runner-e2e verify-rust build-release-artifacts verify-release-artifacts verify-workflows

rust-fmt:
	./tools/rust-ci.sh fmt

rust-clippy:
	./tools/rust-ci.sh clippy

rust-test:
	./tools/rust-ci.sh test

wasm-check:
	./tools/rust-ci.sh wasm

runner-e2e:
	./tools/rust-ci.sh e2e

verify-rust:
	./tools/rust-ci.sh verify

build-release-artifacts:
	./tools/release-packager/package.sh "$${RELEASE_VERSION:-dev}" "$${RELEASE_OUTPUT_DIR:-dist}"

verify-release-artifacts:
	AI_ARENA_DIR="$${AI_ARENA_DIR:?set AI_ARENA_DIR to the pinned ai-arena checkout}" ./tools/release-packager/verify.sh "$${RELEASE_VERSION:-dev}" "$${RELEASE_OUTPUT_DIR:-dist}"

verify-workflows:
	./tools/workflow-lint.sh --mode=ci
