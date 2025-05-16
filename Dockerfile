FROM rust:bookworm

RUN rustup toolchain install stable
RUN rustup default stable
RUN rustup target add aarch64-unknown-linux-gnu
RUN cargo install cargo-deb

RUN dpkg --add-architecture arm64
# Adding bookworm-backports to make ubuntu-keyring package available which is needed by multistrap.conf
RUN sed -i 's/bookworm-updates/bookworm-updates bookworm-backports/g' /etc/apt/sources.list.d/debian.sources

# install libc6-dev:arm64 because rust doesn't pickup the multistrap version or has some conflicts there
RUN apt update && apt install -y multistrap crossbuild-essential-arm64

COPY ./multistrap.conf .
RUN multistrap -f multistrap.conf -d /tmp/aarch64

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/tmp/aarch64/usr/share/pkgconfig
ENV PKG_CONFIG_LIBDIR=/tmp/aarch64/usr/lib/aarch64-linux-gnu/pkgconfig/
ENV PKG_CONFIG_SYSROOT_DIR=/tmp/aarch64/
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/aarch64-linux-gnu-gcc
ENV RUSTFLAGS="-C link-arg=--sysroot=/tmp/aarch64"
ENV CARGO_HOME=/.cargo
ENV CFLAGS="-I/tmp/aarch64/usr/include/aarch64-linux-gnu"


WORKDIR /project/application
CMD cargo build --target aarch64-unknown-linux-gnu --release
