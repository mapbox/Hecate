FROM ubuntu:18.10

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        software-properties-common \
        libcurl4-openssl-dev \
        apt-transport-https \
        postgresql-contrib \
        python-setuptools \
        build-essential \
        libiberty-dev \
        binutils-dev \
        pkg-config \
        zlib1g-dev \
        postgresql \
        python-dev \
        libssl-dev \
        libelf-dev \
        libdw-dev \
        locales \
        postgis \
        openssl \
        python \
        unzip \
        cmake \
        curl \
        wget \
        git \
        gcc \
    && locale-gen en_US.UTF-8 \
    && bash -c "echo \"America/New_York\" > /etc/timezone"

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-12-01

RUN echo "local all all trust " > /etc/postgresql/10/main/pg_hba.conf \
    && echo "host all all 127.0.0.1/32 trust" >> /etc/postgresql/10/main/pg_hba.conf \
    && echo "host all all ::1/128 trust" >> /etc/postgresql/10/main/pg_hba.conf

RUN wget 'https://github.com/opengeospatial/ets-wfs20/archive/master.zip' \
    && unzip master

WORKDIR /usr/local/src/hecate
ADD . /usr/local/src/hecate

CMD ./tests/test.sh
