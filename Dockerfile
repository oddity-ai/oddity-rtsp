# syntax=docker/dockerfile:1
FROM --platform=amd64 rust:1.66-bullseye as build

RUN apt update && \
    apt install ffmpeg clang libavutil-dev libavformat-dev libavfilter-dev libavdevice-dev libswresample-dev libswscale-dev libavcodec-dev -y

WORKDIR /work

COPY ./ ./

# build for release
RUN cd oddity-rtsp-server && \
    cargo build --release

FROM --platform=amd64 debian:bullseye-slim

ENV LOG="oddity_rtsp_server=info"

RUN apt update && \
    apt install ffmpeg  -y

# copy the build artifact from the build stage
COPY --from=build /work/oddity-rtsp-server/target/release/oddity-rtsp-server /

# set the startup command to run your binary
CMD ["/oddity-rtsp-server"]