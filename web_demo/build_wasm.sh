WASM_FILE=micro_ear_wasm.wasm
TARGET_DIR=../web_demo/wasm

cd ../micro_ear_wasm

cargo build --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/$WASM_FILE $TARGET_DIR
wasm-strip $TARGET_DIR/$WASM_FILE