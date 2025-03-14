section .note.GNU-stack noalloc noexec nowrite progbits

FALIGN    equ  1 << 0            ; align loaded modules on page boundaries
FMEMINFO  equ  1 << 1            ; provide memory map
;FVIDMODE  equ  1 << 2            ; try to set graphics mode
FLAGS     equ  FALIGN | FMEMINFO; | FVIDMODE
MAGIC     equ  0x1BADB002
CHECKSUM  equ -(MAGIC + FLAGS)

section .multiboot.data
align 4
    ; header
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

global start
extern start64

section .bss
alignb 4096


section .rodata
alignb 8

; Implement gdt here

section .multiboot.text
bits 32
	global _start
	extern _kernel_start
	extern _kernel_end
	extern _kernel_higher

section .text
bits 64

; The actual code
_start:
	jmp start64
