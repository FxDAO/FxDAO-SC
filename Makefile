default: build

test: build
	cargo test
	cargo test --features testutils

test-optimized: build-optimized
	cargo test
	cargo test --features testutils

build:
	cargo build --target wasm32-unknown-unknown --release -p vaults
	cd target/wasm32-unknown-unknown/release/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

build-optimized:
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p vaults
	cd target/wasm32-unknown-unknown/release/ && \
		for i in *.wasm ; do \
			wasm-opt -Oz "$$i" -o "$$i.tmp" && mv "$$i.tmp" "$$i"; \
			ls -l "$$i"; \
		done

watch:
	cargo watch --clear --watch-when-idle --shell '$(MAKE)'

fmt:
	cargo fmt --all

clean:
	cargo clean