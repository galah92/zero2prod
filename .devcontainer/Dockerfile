FROM mcr.microsoft.com/devcontainers/rust:0-1-bullseye AS dev
WORKDIR /app
RUN apt-get update -y && export DEBIAN_FRONTEND=noninteractive && apt-get install -y lld clang
