#pragma once

#include "stddef.h"
#include "stdarg.h"

#define SEEK_END 3
#define SEEK_SET 4

//

typedef struct FILE {} FILE;

//

extern FILE* stderr;
extern FILE* stdout;

//

extern int fprintf(FILE* stream, const char* format, ...);

extern int printf(const char* format, ...);

extern int vsnprintf(char* s, size_t n, const char* format, va_list args);

extern int snprintf(char* s, size_t n, const char* format, ...);

extern int sscanf(const char* s, const char* format, ...);

extern FILE* fopen(const char* filename, const char* mode);

extern size_t fread(void* ptr, size_t size, size_t count, FILE* stream);

extern size_t fwrite(const void* ptr, size_t size, size_t count, FILE* stream);

extern int fseek(FILE* stream, long int offset, int origin);

extern int ftell(FILE* stream);

