compile:
	cargo run > result.asm

assemble: compile
	nasm -f elf64 result.asm

link: assemble
	gcc result.o -o result -lm

run: link
	./result
