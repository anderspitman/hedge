#include "eri_sdk.h"

__attribute__((import_module("hedge")))
__attribute__((import_name("open")))
i32 hedge_open_import(i32 ptr, i32 size);

__attribute__((import_module("hedge")))
__attribute__((import_name("write")))
u32 hedge_write_import(i32 fd, i32 ptr, i32 size);

i32 erisdk_open(const char *path) {
    return hedge_open_import((i32)path, erisdk_cstr_len(path));
}

u32 erisdk_write(i32 handle, const u8 *bytes, u32 size) {
    return hedge_write_import(handle, (i32)bytes, size);
}
