FROM rust:1.82-bookworm

# 仅安装跨编译所需的 mingw-w64 工具链
RUN apt-get update \
    && apt-get install -y --no-install-recommends mingw-w64 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /work/update-toolkit
