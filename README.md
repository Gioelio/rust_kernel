# A SIMPLE RUST KERNEL

This project is a minimal Rust-based kernel designed for x86 systems, capable 
of execution via QEMU emulation. Below are the steps required to build and 
emulate the kernel.

## Purpose

The primary purpose of this kernel is to serve as an educational tool,
demonstrating how Rust can be utilized for kernel development from the 
ground up.

## Features

Currently, the kernel is basic, featuring simple VGA interaction to print a 
hardcoded string to the screen. Planned features include:
- Memory allocation
- Support to libc library


## Build and emulate

The simplest way to compile the kernel is by using Docker. However, it is also 
possible to compile the project directly. The recommended environment is Linux,
as the procedure may differ on other operating systems.

### Building using docker

*Dependencies required*:
- Docker
- QEMU (qemu-system-x86)

*Commands*:

To build the kernel using Docker, execute:

```
    docker compose up --build
```

To extract the binary files to the current directory, use:

```
    docker cp rust_os-kernel-1:/kernel/bin .
```

### Building from a Linux Machine

Building from source requires at least Rust to be installed. Additional 
dependencies may vary depending on the Linux distribution. The tested 
distributions are listed below:

#### Debian

*Dependencies installation*:

```
    sudo apt update && sudo apt install build-essential \
	grub-pc-bin xorriso qemu-system-x86 nasm
```
#### Archlinux

*Dependecies installation*:
```
    pacman -Syyu --noconfirm make nasm xorriso gcc grub efibootmgr mtools
```

### Compilation and artifacts

To compile the code use:

```
    make build-iso
```

To compile and start the QEMU emulation use:

```
    make
```

Both commands create a /bin directory where the compiled files are stored. 
The ISO file is the bootable file with multiboot support. 
The .bin32 file is the kernel, which can be emulated using: 
`qemu-system-x86 --kernel kernel.bin32` (from the bin directory).


