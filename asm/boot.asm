section .note.GNU-stack noalloc noexec nowrite progbits

; Multiboot header constants
MBALIGN  equ  1 << 0            ; align loaded modules on page boundaries
MEMINFO  equ  1 << 1            ; provide memory map
FLAGS    equ  MBALIGN | MEMINFO ; this is the Multiboot 'flag' field
MAGIC    equ  0x1BADB002        ; 'magic number' lets bootloader find the header
CHECKSUM equ -(MAGIC + FLAGS)   ; checksum of above, to prove we are multiboot

; The multiboot header must be in the first 8192 bytes of the kernel file
; and must be aligned on a 4-byte boundary
section .multiboot
align 4
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

section .bss
alignb 4096
; Stack space
stack_bottom:
    resb 16384  ; 16 KB stack
stack_top:

; Page tables for initial 64-bit setup
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

section .rodata
align 8
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) ; data segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .multiboot.text
bits 32
global _start
extern start64

_start:
    ; Set up stack
    mov esp, stack_top
    
    ; Set up page tables for 64-bit mode
    ; Map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax
    
    ; Map first P3 entry to P2 table
    mov eax, p2_table
    or eax, 0b11 ; present + writable
    mov [p3_table], eax
    
    ; Map each P2 entry to a huge 2MiB page
    mov ecx, 0
.map_p2_table:
    ; Map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page
    or eax, 0b10000011 ; present + writable + huge
    mov [p2_table + ecx * 8], eax ; map ecx-th entry
    
    inc ecx            ; increase counter
    cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
    jne .map_p2_table  ; else map the next entry

    ; Enable PAE-flag in cr4
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Load P4 to cr3 register
    mov eax, p4_table
    mov cr3, eax

    ; Set the long mode bit in the EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ; Load GDT
    lgdt [gdt64.pointer]
    
    ; Jump to 64-bit code
    jmp gdt64.code:long_mode_start

section .text
bits 64
long_mode_start:
    ; Load data segment
    mov ax, gdt64.data
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Call Rust entry point
    call start64
    
    ; Halt if we return
    cli
.halt:
    hlt
    jmp .halt
