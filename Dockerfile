FROM ubuntu:16.04

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y && \
    apt-get install -y \
        locales && \
        locale-gen en_US.UTF-8 && \
        bash -c "echo \"America/New_York\" > /etc/timezone"

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN apt-get install -y curl wget libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc binutils-dev libiberty-dev git build-essential

RUN git clone http://github.com/SimonKagstrom/kcov.git && \
    cd kcov && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install

RUN curl https://sh.rustup.rs -sSf | sh
RUN cargo install cargo-kcov

