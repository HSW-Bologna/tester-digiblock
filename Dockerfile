#FROM ghcr.io/slint-ui/slint/armv7-unknown-linux-gnueabihf:latest
#FROM ghcr.io/slint-ui/slint/aarch64-unknown-linux-gnu:latest
FROM ghcr.io/iced-rs/armv7:latest

RUN apt update
RUN apt upgrade -y
RUN apt install -y libudev-dev:armhf
