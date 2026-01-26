FROM ubuntu:noble

RUN apt update
RUN apt install -y sudo
RUN useradd -m -s /bin/bash -G sudo admin \
	&& echo "admin:admin" | chpasswd \
	&& echo "admin ALL=(ALL:ALL) NOPASSWD:ALL" >> /etc/sudoers
USER admin
WORKDIR /home/admin
RUN sudo apt-get install -y vim net-tools wget curl git openssh-server
# 安装Solana
RUN sudo apt-get install -y \
    build-essential \
    pkg-config \
    libudev-dev llvm libclang-dev \
    protobuf-compiler libssl-dev
RUN curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
