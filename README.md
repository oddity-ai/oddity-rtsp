<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://oddity.ai/img/logo_full_light.png">
    <source media="(prefers-color-scheme: light)" srcset="https://oddity.ai/img/logo_full.png">
    <img alt="Oddity.ai" src="https://oddity.ai/img/logo_full.png" height="45px">
  </picture>
  <br/>
  <h1 align="center">RTSP Server</h1>
  <p align="center">RTSP server built in Rust.</p>
</p>

## ⚡ Quickstart

Download, compile (with Cargo) and run the RTSP server with a simple configuration file:

```sh
git clone git@github.com:oddity-ai/oddity-rtsp.git
cd oddity-rtsp
cd oddity-rtsp-server
echo 'server:
  host: 0.0.0.0
  port: 5554
media:
  - name: "Big Buck Bunny"
    path: "/example"
    kind: file
    source: "https://storage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"' >> config.yaml
LOG=oddity_rtsp_server=info cargo run --release -- config.yaml
```

Play the file using `ffmpeg`:

```sh
ffplay "rtsp://localhost:5554/example"
```

Refer to the [configuration](#-configuration) section for more information
on how to set it up.

## ✅ Features

* Broadcast a single input stream to multiple clients.
* Play video files on repeat, and broadcast them as if they were a stream.
* RTSP RFC 2326 compliant.
* RTSP over TCP in interleaved mode.

Not supported:
* RTSP over UDP. Only RTSP over TCP (interleaved) is supported right now.

## 📖 Summary

This repository holds a RFC 2326 compliant implementation of an RTSP server that
can function as an RTSP proxy or restreamer.

Using this server, you can have a single input stream, and distribute it to
unlimited clients over RTSP. This can be useful to circumvent the bandwidth
limitation of a security camera for example. You can also add a video file as
a source, and any clients will see the file on repeat, as if it were a live
stream.

```
Sources:                                                 Sessions:

                                                          🖥 ️
┌──────────────────────┐                                 ┌────────────────┐
│ ~/myvideo.mp4        ├───┐                        ┌───►│ PLAY /video/1  │
└──────────────────────┘   │                        │    └────────────────┘
                           │                        │     💻
┌──────────────────────┐   │    ┌───────────────┐   │    ┌────────────────┐
│ rtsp://mystream.net/ ├───┼───►│  RTSP SERVER  ├───┼───►│ PLAY /stream/1 │
└──────────────────────┘   │    └───────────────┘   │    └────────────────┘
                           │                        │     🖥️
┌──────────────────────┐   │                        │    ┌────────────────┐
│ http://mystream.com/ ├───┘                        └───►│ PLAY /stream/1 │
└──────────────────────┘                                 └────────────────┘
```

## 🔧 Configuration

### Configuration File

The first and only argument the server expects is the location of the configuration
file:

```sh
oddity-rtsp-server /path/to/config.yaml
```

The configuration file is a YAML file, that should look something like this:

#### `config.yaml`

```yaml
server:
  host: 0.0.0.0
  port: 554

media:
  - name: "Name of Source"
    path: "/url/to/source"
    kind: file
    source: "/path/to/file.mp4"
  - name: "Name of Another Source"
    path: "/url/to/other/source"
    kind: stream
    source: "rtsp://10.0.0.1/stream"
```

In the above example, two sources are configured:

* A `file` source that points to the local file in `/path/to/file.mp4`. This can
  also be a URL in some cases, as long as the underlying media is not a streaming
  source, but a file source (a source that supports seeking). Clients can connect
  to the stream at path `rtsp://server/url/to/source`.

* A `stream` source that points to a different RTSP stream, reachable at the path
  `rtsp://server/url/to/other/source`. Note that regardless of how many clients
  connect to the stream, the server will only have a single stream open to the
  original RTSP source.

Note: To run the above example, the server must be called with superuser priviliges,
because it uses a protected port (554):

```sh
sudo LOG=oddity_rtsp_server=info ./oddity-rtsp-server
```

### Logging

Use the `LOG` environment variable to control what will be logged to the console.
To display all informational messages, run the server as follows:

```sh
LOG=oddity_rtsp_server=info oddity-rtsp-server
```

When debugging, it might be useful to display tracing messages as well. Use the
following setting to display tracing messages produced by the server:

```sh
LOG=oddity_rtsp_server=trace oddity-rtsp-server
```

You can also display log messages from the ffmpeg backend, like so:

```sh
LOG=oddity_rtsp_server=trace,video=trace oddity-rtsp-server
```

Or simply enable all tracing messages:

```sh
LOG=trace oddity-rtsp-server
```

## 📦 Crates

The repo consists of a number of crates, each with their own specific function:

* `oddity-rtsp-server`: RTSP server implementation. This is the application
  crate, the one that runs the actual server. It depends on the library crates.

* `oddity-rtsp-protocol`: Parsing and serialization for the RTSP protocol.

* `oddity-sdp-protocol`: Parsing and serialization for the SDP protocol.

Building these crates from sources requires that both `clang` and `libavdevice`
are available for linking against on your system.

## ✨ Credits

`oddity-rtsp` only exists thanks to the following organizations and people:

* All [contributors](https://github.com/oddity-ai/oddity-rtsp/graphs/contributors) for their work!
* Everyone who worked on [video-rs](https://github.com/oddity-ai/video-rs) since `oddity-rtsp` depends heavily on `video-rs`.
* Multiple unnamed customers and integrators that helped us develop and troubleshoot the RTSP server and make it ready for production use 💪.
* [Provincie Utrecht](https://www.provincie-utrecht.nl/) for supporting this project as part of the "Situational Awareness Software" project.
* [zmwangx](https://github.com/zmwangx) for maintaining [rust-ffmpeg](https://github.com/zmwangx/rust-ffmpeg).
* The [FFmpeg project](https://ffmpeg.org/) for `ffmpeg` and the `ffmpeg` libraries.

## ⚖️ License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
