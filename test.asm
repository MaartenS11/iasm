jmp main

fibonacci:
    load esi, [esp + 4]
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
    load eax, 5
    push eax
    call fibonacci

    load eax, 6
    push eax
    call fibonacci

    ;load eax, 45
    ;push eax
    ;call fibonacci
