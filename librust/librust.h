
extern "C" {

/// Write a null-terminated C string to the terminal. Returns `false` on failure.
bool print(const uint8_t *s);

/// Minimal printf implementation supporting `%c`, `%s`, and `%%`.
///
/// # Safety
/// The caller must ensure that variadic arguments match the format specifiers.
int32_t printf(const uint8_t *format, ...);

int32_t puts(const uint8_t *s);

uint8_t putchar(uint8_t ic);

uint8_t *memmove(uint8_t *dest, const uint8_t *src, uintptr_t n);

uintptr_t strlen(const uint8_t *s);

uint8_t *memset(uint8_t *s, uint8_t c, uintptr_t n);

uint8_t *memcpy(uint8_t *dest, const uint8_t *src, uintptr_t n);

int32_t memcmp(const uint8_t *s1, const uint8_t *s2, uintptr_t n);

int32_t bcmp(const uint8_t *s1, const uint8_t *s2, uintptr_t n);

}  // extern "C"
