# Chromecast Link Tool

[![Build Status](https://travis-ci.org/azasypkin/chromecast-link-tool.svg?branch=master)](https://travis-ci.org/azasypkin/chromecast-link-tool)

Just a helper tool for Rust [Chromecast Link](https://github.com/azasypkin/chromecast-link) crate.

## Usage
```bash
$ cargo run -- -a chromecast.local --stream http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4 --media-type video/mp4
$ cargo run -- -a chromecast.local --stream http://xxx.mp4 --stream-type buffered
```