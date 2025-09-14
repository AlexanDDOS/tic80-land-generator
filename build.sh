rustup target add wasm32-unknown-unknown
cargo build --release --target=wasm32-unknown-unknown
tic80 --fs . --cmd "new wasm & import binary target/wasm32-unknown-unknown/release/cart.wasm & save cart"
