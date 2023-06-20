FROM ghcr.io/slint-ui/slint/armv7-unknown-linux-gnueabihf:latest
RUN apt update
RUN apt upgrade -y
RUN apt install -y libudev-dev:armhf
