#include "syscalls.h"

void write(int fd, void *buf, int count) {
    asm("mv a0, %0" : : "r" (fd));
    asm("mv a1, %0" : : "r" (buf));
    asm("mv a2, %0" : : "r" (count));
    asm("li a7, 4");
    asm("ecall");
}

void brk(void *addr) {
    asm("mv a0, %0" : : "r" (addr));
    asm("li a7, 45");
    asm("ecall");
}

size_t read(int fd, void *buf, int count) {
    register int return_value asm("a0") = 0;

    asm("mv a0, %0" : : "r" (fd));
    asm("mv a1, %0" : : "r" (buf));
    asm("mv a2, %0" : : "r" (count));
    asm("li a7, 3");
    asm("ecall" : "=r" (return_value));
    return return_value;
}
