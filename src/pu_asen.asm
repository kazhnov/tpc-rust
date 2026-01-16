format ELF64
	
extrn lawa	
public _start
_start:
    call lawa
    mov rdi, rax
    mov rax, 60
    syscall
	
public __tp_exit
__tp_exit:	
	mov rax, 60
	syscall
	ret
	
