# Interpreted Assembly
A small RISC-V virtual machine written in Rust capable of running textual
assembly files produced by GCC. The interpreter also emulates some Linux system
calls such as the read, write and brk syscalls. These are needed for reading
and writing to stdin and stdout, brk is used for dynamic memory allocation.

Does not support all instructions or registers, just enough to run simple C
programs that have basic input output using stdin/stdout, loops, arrays,
integers, strings, structs, dynamic memory allocation, ... One of the main
things not supported currently is floating point calculations.


