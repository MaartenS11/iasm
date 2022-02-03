j main

fibonacci:
    lw a5, 0(sp)
    li a0, 0
    li a2, 0
    li a3, 1
.loop:
    mv a1, a3
    add a3, a3, a2
    mv a2, a1
    addi a0, a0, 1
    bne a0, a5, .loop
    mv a0, a3
    ret

squares:
    mv a4, sp
    li a1, 48
    sub sp, a1
    
    mv a2, sp
    add a2, a2, a1
    mv a3, sp
    li a0, 2
.L1:
    sw a0, 0(a3)
    add a0, a0, a0
    addi a3, a3, 4
    bne a3, a2, .L1 #Check iteration count
    
    mv sp, a4
    ret

main:
    # Calculate the (5 + 1)th fibonacci number
    li a0, 5
    addi sp, sp, -4
    sw a0, 0(sp)
    jal ra, fibonacci
    addi sp, sp, 4

    # Calculate the (6 + 1)th fibonacci number
    li a0, 6
    addi sp, sp, -4
    sw a0, 0(sp)
    jal ra, fibonacci
    addi sp, sp, 4

    # Calculate the (45 + 1)th fibonacci number
    li a0, 45
    addi sp, sp, -4
    sw a0, 0(sp)
    jal ra, fibonacci
    addi sp, sp, 4

    # Generate table containing squares of 2
    # 0x10│2
    # 0x14│4
    # 0x18│8
    # 0x1c│16
    # 0x20│32
    # 0x24│64
    # 0x28│128
    # 0x2c│256
    # 0x30│512
    # 0x34│1024
    # 0x38│2048
    # 0x3c│4096
    jal ra, squares
