FROM ubuntu:18.04

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y \
    && apt-get install -y wget gnupg2 \
    && wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
    && echo "deb http://apt.postgresql.org/pub/repos/apt/ bionic"-pgdg main | tee /etc/apt/sources.list.d/pgdg.list \
    && apt-get update -y \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        software-properties-common \
        libcurl4-openssl-dev \
        apt-transport-https \
        postgresql-contrib-11 \
        build-essential \
        libiberty-dev \
        binutils-dev \
        pkg-config \
        zlib1g-dev \
        postgresql-11 \
        postgresql-11-postgis-2.5 \
        libssl-dev \
        libelf-dev \
        libdw-dev \
        locales \
        postgis \
        openssl \
        cmake \
        curl \
        wget \
        git \
        gcc \
        git \
    && locale-gen en_US.UTF-8 \
    && bash -c "echo \"America/New_York\" > /etc/timezone"

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN curl 'https://nodejs.org/dist/v10.15.3/node-v10.15.3-linux-x64.tar.gz' | tar -xzv \
    && cp ./node-v10.15.3-linux-x64/bin/node /usr/bin/ \
    && ./node-v10.15.3-linux-x64/bin/npm install -g npm \
    && npm install -g yarn \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.40.0  \
    && echo "local all all trust " > /etc/postgresql/11/main/pg_hba.conf \
    && echo "host all all 127.0.0.1/32 trust" >> /etc/postgresql/11/main/pg_hba.conf \
    && echo "host all all ::1/128 trust" >> /etc/postgresql/11/main/pg_hba.conf

WORKDIR /usr/local/src/hecate
ADD . /usr/local/src/hecate

CMD ./test.sh
