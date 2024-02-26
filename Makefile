default: build

test: build
	cargo test
	cargo test --features testutils

test-optimized: build-optimized
	cargo test
	cargo test --features testutils

build:
	soroban contract build --package vaults
	soroban contract build

build-optimized:
	soroban contract build --package vaults
	soroban contract build
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/vaults.wasm --wasm-out ./target/wasm32-unknown-unknown/release/vaults.wasm
	soroban contract optimize --wasm ./target/wasm32-unknown-unknown/release/stable_liquidity_pool.wasm --wasm-out ./target/wasm32-unknown-unknown/release/stable_liquidity_pool.wasm

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
      stellar/quickstart:testing@sha256:3c7947f65db493f2ab8ca639753130ba4916c57d000d4a1f01ec530e3423853b \
      --standalone \
      --enable-soroban-rpc

standalone_fund_accounts:
	curl "http://localhost:8000/friendbot?addr=GCQDSTJQKHSZICYDHGI3U73VOYCPGI5QEEOCVUORFWCYN5MH26XHH2LZ" && \
	curl "http://localhost:8000/friendbot?addr=GBMHIX37J3IZC4H2TVOQ6RKYGLCNNX543NU3OI3SP4LDBERVCO3DCCOD" && \
	curl "http://localhost:8000/friendbot?addr=GAZ5H54I4O7QF64HBLVWWAPDZ7OYRI3EGMJ27YJGSTBE2L7VQNNEIWZF" && \
	curl "http://localhost:8000/friendbot?addr=GDGMFR44SMGNCWTZFP6YPHBOX2IYNY7WQGCBZQBTU7QYKPLP4V7BG4NI" && \
	curl "http://localhost:8000/friendbot?addr=GDPOWRFN5CZXNSPTYOSSUTKRFZ23MOZBMFA2H2Q4ACIL62QNILIDWWSU" && \
	curl "http://localhost:8000/friendbot?addr=GDE3RXHI2IQKFAOFC23GZIGR6FMQD2GKQ2IUJDIN3JGH4Z5COIZWGT2A"


