FROM rust:1.84-slim-bullseye AS build

RUN apt-get update && apt-get install -y \
    build-essential \
	grub-pc-bin \
	xorriso \
	nasm 

WORKDIR /kernel

COPY . .

RUN make build-iso

