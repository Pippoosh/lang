bits 64
default rel
extern ExitProcess
extern printf
extern scanf
section .data
    fmt_in db "%lf", 0
    fmt_out db "%lf", 10, 0
    fmt_str db "%s", 10, 0
    prompt db "Enter a number: ", 0
    str_0 db "Enter a number:", 0
section .bss
    vars resq 100
section .text
global main
main:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    lea rcx, [fmt_str]
    lea rdx, [str_0]
    xor rax, rax
    call printf
    add rsp, 32
    sub rsp, 32
    lea rcx, [fmt_out]
    xor rax, rax
    call printf
    add rsp, 32
    sub rsp, 32
    lea rcx, [prompt]
    xor rax, rax
    call printf
    lea rdx, [vars + 0]
    lea rcx, [fmt_in]
    xor rax, rax
    call scanf
    add rsp, 32
    mov rax, 100000
    mov [vars + 8], rax
.L0:
    mov rax, 600000
    push rax
    mov rax, [vars + 8]
    pop rcx
    cmp rax, rcx
    jg .L1
    mov rax, 100000
    add [vars + 8], rax
    jmp .L0
.L1:
    mov rax, [vars + 8]
    push rax
    mov rax, [vars + 0]
    pop rcx
    imul rcx
    mov rcx, 100000
    cqo
    idiv rcx
    sub rsp, 32
    movq xmm0, rax
    lea rcx, [fmt_out]
    mov rax, 1
    call printf
    add rsp, 32
    sub rsp, 32
    lea rcx, [fmt_out]
    xor rax, rax
    call printf
    add rsp, 32
    mov rcx, 0
    call ExitProcess
    mov rcx, 0
    call ExitProcess
