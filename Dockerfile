# Build
FROM clux/muslrust:1.70.0 as build


COPY . /workdir

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /workdir
RUN RUSTFLAGS=-Cforce-frame-pointers=yes \
  cargo build \
    --profile=container \
    --target=x86_64-unknown-linux-musl
RUN objcopy \
  --compress-debug-sections \
  /workdir/target/x86_64-unknown-linux-musl/container/radicle-ci \
  /workdir/target/x86_64-unknown-linux-musl/container/radicle-ci.compressed

# Run
FROM debian:bullseye-slim@sha256:25f10b4f1ded5341a3ca0a30290ff3cd5639415f0c5a2222d5e7d5dd72952aa1

RUN echo deb http://deb.debian.org/debian bullseye-backports main contrib non-free >/etc/apt/sources.list.d/backports.list
RUN apt-get update && \
  apt -t bullseye-backports install --yes git && \
  rm -rf /var/lib/apt/lists/*
COPY --from=build \
  /workdir/target/x86_64-unknown-linux-musl/container/radicle-ci.compressed \
  /usr/local/bin/radicle-ci

WORKDIR /app

ENTRYPOINT ["/usr/local/bin/radicle-ci", "--concourse-url=$CONCOURSE_URL", "--concourse-user=$CONCOURSE_USER", "--concourse-pass=$CONCOURSE_PASS", "--radicle-api-url=$RADICLE_API_URL"]