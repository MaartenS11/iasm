jmp main

fibonacci:
    ld esi, [esp + 4]
    li eax, 0
    li ecx, 0
    li edx, 1
.loop:
    mov ebx, edx
    add edx, ecx
    mov ecx, ebx
    inc eax
    cmp eax, esi
    jne .loop
    mov eax, edx
    ret

main:
    li eax, 5
    push eax
    call fibonacci

    li eax, 6
    push eax
    call fibonacci

    ;li eax, 45
    ;push eax
    ;call fibonacci
