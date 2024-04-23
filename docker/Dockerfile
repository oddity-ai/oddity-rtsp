# Create environment for build rust project
# Official rust image have old library versions, because it is based
# debian image. So using ubuntu:latest image
FROM ubuntu:latest as rust_ubuntu

USER root
ENV USER root

# Install package dependencies.
RUN apt-get update \
    && apt-get install -y \
    apt-utils \
    curl \
    gcc \
    && rm -rf /var/lib/apt/lists/*
# Install Rust
RUN curl https://sh.rustup.rs -sSf > /tmp/rustup-init.sh \
    && chmod +x /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh -y \
    && rm -rf /tmp/rustup-init.sh

# Update the local crate index
RUN ~/.cargo/bin/cargo search

FROM rust_ubuntu as builder
ENV PATH "$PATH:/root/.cargo/bin"
# Update image and install dependecies
RUN apt update && apt upgrade -y && apt install -y tree \
                        ffmpeg  \
                        clang libavcodec-dev libavformat-dev \
                        libavutil-dev libavfilter-dev \
                        libavdevice-dev pkg-config

# set up work directory
WORKDIR /usr/src/oddity-rtsp
# copy files for building
COPY ./oddity-rtsp-protocol ./oddity-rtsp-protocol
COPY ./oddity-sdp-protocol ./oddity-sdp-protocol
COPY ./oddity-rtsp-server ./oddity-rtsp-server
COPY ./Cargo.lock ./
COPY ./Cargo.toml ./
# build rtsp-server
RUN cargo build --release
RUN tree ./target
# Create new image for serving project
FROM ubuntu:latest as release
# install dependencies for run application
RUN apt update && apt upgrade -y && apt install -y ffmpeg
# copy files from builder image
COPY --from=builder /usr/src/oddity-rtsp/target/release/oddity-rtsp-server /usr/local/bin/oddity-rtsp-server
RUN rm -rf /var/lib/apt/lists/*
# create directory and copy config.yaml
WORKDIR /server
COPY ./docker/config-example.yaml /server/config.yaml
# create directory for local files
RUN mkdir /server/upload
# open port for transmit
EXPOSE 5554

CMD ["oddity-rtsp-server", "/server/config.yaml"]
