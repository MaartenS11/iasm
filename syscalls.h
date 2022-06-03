#pragma once
#define NULL 0
typedef unsigned long size_t;

void write(int fd, void *buf, int count);
void brk(void *addr);
size_t read(int fd, void *buf, int count);
