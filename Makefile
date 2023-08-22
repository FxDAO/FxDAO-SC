default: build

test: build
	cargo test
	cargo test --features testutils

test-optimized: build-optimized
	cargo test
	cargo test --features testutils

build:
	cargo rustc --crate-type cdylib --target wasm32-unknown-unknown --release --package vaults
	cargo rustc --crate-type cdylib --target wasm32-unknown-unknown --release --package safety-pool
	cargo rustc --crate-type cdylib --target wasm32-unknown-unknown --release --package governance
	cargo rustc --crate-type cdylib --target wasm32-unknown-unknown --release --package stable-liquidity-pool
	cd target/wasm32-unknown-unknown/release/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

build-optimized:
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p vaults
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p safety-pool
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p governance
	cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort -p stable-liquidity-pool
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

sandbox_full_set_up:
	make sandbox_install_contracts
	make sandbox_deploy_assets

launch_standalone:
	docker run -d -it \
      -p 8000:8000 \
      --name stellar-soroban-network \
      stellar/quickstart:soroban-dev@sha256:8a99332f834ca82e3ac1418143736af59b5288e792d1c4278d6c547c6ed8da3b \
      --standalone \
      --enable-soroban-rpc

standalone_fund_accounts:
	curl "http://localhost:8000/friendbot?addr=GCQDSTJQKHSZICYDHGI3U73VOYCPGI5QEEOCVUORFWCYN5MH26XHH2LZ" && \
	curl "http://localhost:8000/friendbot?addr=GBMHIX37J3IZC4H2TVOQ6RKYGLCNNX543NU3OI3SP4LDBERVCO3DCCOD" && \
	curl "http://localhost:8000/friendbot?addr=GAZ5H54I4O7QF64HBLVWWAPDZ7OYRI3EGMJ27YJGSTBE2L7VQNNEIWZF" && \
	curl "http://localhost:8000/friendbot?addr=GDGMFR44SMGNCWTZFP6YPHBOX2IYNY7WQGCBZQBTU7QYKPLP4V7BG4NI" && \
	curl "http://localhost:8000/friendbot?addr=GDPOWRFN5CZXNSPTYOSSUTKRFZ23MOZBMFA2H2Q4ACIL62QNILIDWWSU" && \
	curl "http://localhost:8000/friendbot?addr=GDE3RXHI2IQKFAOFC23GZIGR6FMQD2GKQ2IUJDIN3JGH4Z5COIZWGT2A"


