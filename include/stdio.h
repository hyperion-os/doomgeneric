#pragma once

#include "stddef.h"
#include "stdarg.h"

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

// #define	EPERM   1
// #define	ENOENT  2
// #define	ESRCH   3
// #define	EINTR   4
// #define	EIO     5
// #define	ENXIO   6
// #define	E2BIG   7
// #define	ENOEXEC 8
// #define	EBADF   9
// #define	ECHILD  10
// #define	EAGAIN  11
// #define	ENOMEM  12
// #define	EACCES  13
// #define	EFAULT  14
// #define	ENOTBLK 15
// #define	EBUSY   16
// #define	EEXIST  17
// #define	EXDEV   18
// #define	ENODEV  19
// #define	ENOTDIR 20
#define	EISDIR  21
// #define	EINVAL  22
// #define	ENFILE  23
// #define	EMFILE  24
// #define	ENOTTY  25
// #define	ETXTBSY 26
// #define	EFBIG   27
// #define	ENOSPC  28
// #define	ESPIPE  29
// #define	EROFS   30
// #define	EMLINK  31
// #define	EPIPE   32
// #define	EDOM    33
// #define	ERANGE  34

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

