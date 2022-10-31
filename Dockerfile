# @see https://github.com/rust-lang/docker-rust
FROM alpine:3.16 as builder

# install utilities
RUN apk add --update alpine-sdk cmake clang protoc protobuf-dev
RUN apk add --no-cache ca-certificates

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.64.0

RUN set -eux; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
        x86_64) rustArch='x86_64-unknown-linux-musl'; rustupSha256='95427cb0592e32ed39c8bd522fe2a40a746ba07afb8149f91e936cddb4d6eeac' ;; \
        aarch64) rustArch='aarch64-unknown-linux-musl'; rustupSha256='7855404cdc50c20040c743800c947b6f452490d47f8590a4a83bc6f75d1d8eda' ;; \
        *) echo >&2 "unsupported architecture: $apkArch"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.25.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    rustup component add rustfmt;


# build from source
WORKDIR /teritorimon
COPY . /teritorimon
RUN cargo build --release

FROM alpine:3.16
COPY --from=builder /teritorimon/target/release/teritorimon /usr/local/bin/teritorimon
RUN chmod +x /usr/local/bin/teritorimon

# Install ca-certificates
RUN apk add --update ca-certificates

# install utilities
RUN apk add bash

ARG USER_ID
ARG GROUP_ID
ENV HOME /teritorimon
ENV USER_ID ${USER_ID:-1000}
ENV GROUP_ID ${GROUP_ID:-1000}

# add our user and group first
RUN addgroup -g ${GROUP_ID} teritorimon; \
    adduser -D -u ${USER_ID} -G teritorimon -h /teritorimon -s "/bin/bash" teritorimon;

# install su-exec
RUN apk add --no-cache su-exec; \
    su-exec teritorimon true;

WORKDIR /teritorimon

# Update custom cert
COPY certs/* /usr/local/share/ca-certificates/
RUN update-ca-certificates

CMD ["teritorimon"]
