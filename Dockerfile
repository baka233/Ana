FROM ubuntu:16.04 AS build_lrun
COPY judge/externals/lrun /lrun
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    sudo \
    build-essential \
    libseccomp-dev \
    rake \
    g++ && \
    cd /lrun && \
    make

FROM rustlang/rust:nightly-slim AS build_ana
COPY judge/Cargo.toml judge/Cargo.lock /Ana/
COPY judge/src/ /Ana/src
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    libclang-dev \
    clang && \
    cd /Ana && \
    cargo build -v --release

FROM ubuntu:18.04
COPY --from=build_lrun /lrun/src/lrun /usr/local/bin
COPY --from=build_ana /Ana/target/release/ana /usr/local/bin
RUN groupadd -r -g 593 lrun && \
    chown root:lrun /usr/local/bin/lrun && \
    useradd ana && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    sudo \
    libseccomp-dev \
    gcc \
    g++ \
    libzmq3-dev && \
    apt-get clean
EXPOSE 8800
CMD [ "ana" ]