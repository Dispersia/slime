RUSTFLAGS="--cfg=web_sys_unstable_apis" cargo build --target wasm32-unknown-unknown --release \
 && wasm-bindgen --out-dir out --web target/wasm32-unknown-unknown/release/slime.wasm \
 && cp index.html out/index.html