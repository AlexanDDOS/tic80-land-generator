rustup target add wasm32-unknown-unknown
cargo build --release --target=wasm32-unknown-unknown
wasm-opt -Os target/wasm32-unknown-unknown/release/cart.wasm -o cart.wasm
tic80 --fs . --cmd "new wasm & import binary cart.wasm & save cart"
