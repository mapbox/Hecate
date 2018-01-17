FROM ubuntu:17.04

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y \
    && apt-get install -y software-properties-common apt-transport-https python-software-properties python-setuptools python-dev git locales curl postgresql postgresql-contrib wget libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc binutils-dev libiberty-dev git build-essential pkg-config zlib1g-dev python \
    && locale-gen en_US.UTF-8 \
    && bash -c "curl -sS http://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -" \
    && bash -c "echo \"deb http://dl.yarnpkg.com/debian/ stable main\" | tee  /etc/apt/sources.list.d/yarn.list" \
    && bash -c "echo \"America/New_York\" > /etc/timezone" \
    && curl https://nodejs.org/dist/v6.10.2/node-v6.10.2-linux-x64.tar.gz | tar zxC /usr/local --strip-components=1

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN apt-get update && apt-get install -y yarn && yarn install

RUN git clone http://github.com/SimonKagstrom/kcov.git && \
    cd kcov && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="~/.cargo/bin/:${PATH}"

RUN rustup install nightly \
    && rustup default nightly

RUN cargo install cargo-kcov

WORKDIR /usr/local/src/hecate
ADD . /usr/local/src/hecate

CMD yarn test
