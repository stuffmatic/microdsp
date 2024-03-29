PROFILE=$1
# TARGET=thumbv7em-none-eabihf
TARGET=thumbv8m.main-none-eabihf

LIB_NAME=libmicrodsp.rlib
OUTPUT_FILE=disassembly.$PROFILE.$TARGET.txt

if [[ $PROFILE == 'debug' ]]; then
  CARGO_ARGS=""
else
  CARGO_ARGS="--release"
fi

cargo build --target=$TARGET $CARGO_ARGS
arm-none-eabi-objdump -d -S target/$TARGET/$PROFILE/$LIB_NAME | rustfilt > $OUTPUT_FILE
echo Wrote disassembled binary to $OUTPUT_FILE