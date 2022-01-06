load eax, 0
load ecx, 0
load edx, 1
loop:
mov ebx, edx
add edx, ecx
mov ecx, ebx
inc eax
load ebx, 5
cmp eax, ebx
jnz loop
mov eax, edx