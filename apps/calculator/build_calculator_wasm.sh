set -euo pipefail

CC="${WASI_SDK_CC:-../../wasi-sdk/bin/clang}"
CFLAGS="-std=c89 -Wall -pedantic -Wextra -Werror"
${CC} ${CFLAGS} \
    -I ../../eri \
    -o calculator.wasm \
    calculator.c ../../eri/eri_sdk.c ../../eri/eri_sdk_wasm.c \
    --target=wasm32 \
    -nostdlib \
    -Wl,--no-entry \
    -Wl,--export=eri_init \
    -Wl,--export=eri_get_in_msg_buf \
    -Wl,--export=eri_get_out_msg_buf \
    -Wl,--export=eri_update
