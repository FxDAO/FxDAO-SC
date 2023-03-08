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
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p brain
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

sandbox_install_contracts:
	make build
	soroban contract deploy --wasm target/wasm32-unknown-unknown/release/vaults.wasm --id 0000000000000000000000000000000000000000000000000000000000000001

sandbox_deploy_assets:
	# d98fc10ef20b3291ceb69d3170fa7965e98c67ad81983ce6d326cfbe56dfd20a
	soroban lab token wrap --asset native
	# 553b2a327dc588ee541fd3f96163fb920a93bf523bc7cd6b3a7713fd5fc32bb9
	soroban lab token wrap --asset PROTOCOL:GBMHIX37J3IZC4H2TVOQ6RKYGLCNNX543NU3OI3SP4LDBERVCO3DCCOD
	# 53a8fc79bddbb9eddbdf6226018249175de2b947fcddaed2ebee7d06715b834a
	soroban lab token wrap --asset STABLE:GAZ5H54I4O7QF64HBLVWWAPDZ7OYRI3EGMJ27YJGSTBE2L7VQNNEIWZF

sandbox_full_set_up:
	make sandbox_install_contracts
	make sandbox_deploy_assets

