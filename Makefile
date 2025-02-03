.PHONY: run clean

run: bin/kernel.iso
	qemu-system-x86_64 -cdrom $<

clean:
	rm -rf bin/
	rm -rf rust/target/release

bin/folder_creation_hack:
	mkdir -p bin/isodir/boot/grub
	mkdir -p bin/lib/
	touch $@

# Basic Multiboot only understands elf32
bin/kernel.bin32: bin/kernel.bin
	objcopy -I elf64-x86-64 -O elf32-i386 $< $@

bin/boot.o: asm/boot.asm
	nasm -f elf64 -o $@ $<

bin/lib/libkernel.a: $(shell find rust/src/ -type f) bin/folder_creation_hack
	cargo rustc --release --crate-type staticlib --manifest-path rust/Cargo.toml
	cp rust/target/release/libkernel.a bin/lib/libkernel.a
	
bin/kernel.bin: bin/folder_creation_hack bin/boot.o bin/lib/libkernel.a bin/multiboot.o rust/kernel.ld
	ld -n -m elf_x86_64 -o $@ -T rust/kernel.ld bin/boot.o bin/multiboot.o bin/lib/libkernel.a

bin/multiboot.o: asm/multiboot.asm
	nasm -f elf64 -o $@ $<

bin/isodir/boot/grub/grub.cfg: asm/grub.cfg
	mkdir -p bin/isodir/boot/grub/
	cp asm/grub.cfg bin/isodir/boot/grub

bin/isodir/boot/kernel.bin32: bin/kernel.bin32
	cp bin/kernel.bin32 bin/isodir/boot/kernel.bin32

bin/kernel.iso: bin/isodir/boot/kernel.bin32 bin/isodir/boot/grub/grub.cfg
	grub-mkrescue -o bin/kernel.iso bin/isodir

