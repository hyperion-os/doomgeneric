#pragma once

// this is some evil shit
// #define va_start(list, last_arg) (void)((list) = (const void*)(&(last_arg) + 1))

// #define va_end(list) (void)((list) = 0)

// #define va_arg(list, ty) \
//   (( \
//     (const ty*)((list) = (list) + sizeof(ty)) \
//   )[-1])

#define va_start(ap, param) __builtin_va_start(ap, param)
#define va_end(ap)          __builtin_va_end(ap)
#define va_arg(ap, type)    __builtin_va_arg(ap, type)

//

// typedef void* va_list;
typedef __builtin_va_list va_list;
