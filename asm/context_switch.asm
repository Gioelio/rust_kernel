section .text
bits 64

; void switch_context(TaskContext* old, const TaskContext* new)
; RDI = pointer to old context (save current state here)
; RSI = pointer to new context (load new state from here)
global switch_context
switch_context:
    ; Save current task's context to *old
    mov [rdi + 0],  r15
    mov [rdi + 8],  r14
    mov [rdi + 16], r13
    mov [rdi + 24], r12
    mov [rdi + 32], rbx
    mov [rdi + 40], rbp
    
    ; CRITICAL: Save stack pointer (pointing AFTER the return address)
    lea rax, [rsp + 8]      ; RSP after we return
    mov [rdi + 48], rax
    
    ; Save return address (RIP) - where to resume this task
    mov rax, [rsp]
    mov [rdi + 56], rax
    
    ; Load new task's context from *new
    mov r15, [rsi + 0]
    mov r14, [rsi + 8]
    mov r13, [rsi + 16]
    mov r12, [rsi + 24]
    mov rbx, [rsi + 32]
    mov rbp, [rsi + 40]
    
    ; CRITICAL: Load new stack pointer
    mov rsp, [rsi + 48]
    
    ; Load new instruction pointer and jump
    mov rax, [rsi + 56]
	; Enable interrupts
	sti
    jmp rax
