jmp main

fibonacci:
load eax, 0
load ecx, 0
load edx, 1
loop:
mov ebx, edx
add edx, ecx
mov ecx, ebx
inc eax
cmp eax, esi
jnz loop
mov eax, edx
ret

main:
load esi, 5
call fibonacci

load esi, 6
call fibonacci
