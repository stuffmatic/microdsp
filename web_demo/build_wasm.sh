WASM_FILE=microdsp_wrapper.wasm
TARGET_DIR=../web_demo/dist/wasm

cd ../microdsp_wrapper

cargo build --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/$WASM_FILE $TARGET_DIR
wasm-strip $TARGET_DIR/$WASM_FILE
