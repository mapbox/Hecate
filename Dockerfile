FROM ubuntu:18.04

RUN rm /bin/sh && ln -s /bin/bash /bin/sh
ENV SHELL /bin/bash

# set the locale
RUN apt-get update -y \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
        software-properties-common \
        libcurl4-openssl-dev \
        apt-transport-https \
        postgresql-contrib \
        build-essential \
        libiberty-dev \
        openjdk-8-jdk \
        binutils-dev \
        pkg-config \
        zlib1g-dev \
        postgresql \
        libssl-dev \
        libelf-dev \
        libdw-dev \
        locales \
        postgis \
        openssl \
        python \
        maven \
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

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-06-01

RUN echo "local all all trust " > /etc/postgresql/10/main/pg_hba.conf \
    && echo "host all all 127.0.0.1/32 trust" >> /etc/postgresql/10/main/pg_hba.conf \
    && echo "host all all ::1/128 trust" >> /etc/postgresql/10/main/pg_hba.conf

RUN git clone https://github.com/opengeospatial/ets-wfs20.git \
    && cd ets-wfs20 \
    && JAVA_HOME=/usr/lib/jvm/java-1.8.0-openjdk-amd64/ mvn install \
    && echo '<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd"><properties version="1.0"><comment>Test run arguments (ets-wfs20)</comment><entry key="wfs">http://localhost:8000/api/wfs?request=GetCapabilities</entry></properties>' > test-run-props.xml

WORKDIR /usr/local/src/hecate
ADD . /usr/local/src/hecate

CMD ./tests/test.sh
