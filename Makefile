.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-bitcountry-runtime

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all

.PHONY: run
run:
	cargo run --release -- --dev --tmp -lruntime=debug

.PHONY: build
build:
	cargo build --release  --features with-bitcountry-runtime

.PHONY: build-docker
build-docker:
	./scripts/docker_run.sh

.PHONY: run-dev
run-dev:
	./target/release/bitcountry-node --dev --tmp -lruntime=debug

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

.PHONY: githooks
githooks: .git/hooks $(GITHOOKS_DEST)