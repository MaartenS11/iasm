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
pop edx
jmp edx

main:
load esi, 5
mov eax, eip
load ebx, 5
add eax, ebx
push eax
jmp fibonacci

load esi, 6
mov eax, eip
load ebx, 5
add eax, ebx
push eax
jmp fibonacci
