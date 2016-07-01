# Chromecast Link Tool

[![Build Status](https://travis-ci.org/azasypkin/chromecast-link-tool.svg?branch=master)](https://travis-ci.org/azasypkin/chromecast-link-tool)

Just a helper tool for Rust [Chromecast Link](https://github.com/azasypkin/chromecast-link) crate.

## Usage

### Generic features
```bash
// Get some info about the Chromecast-enabled device.
$ cargo run -- -a chromecast.local -i

Number of apps run: 1
App#0: Default Media Receiver (CC1AD845)
Volume level: 1
Muted: false

// Run specific app on the Chromecast.
$ cargo run -- -a chromecast.local -r youtube

// Stop currently active app.
$ cargo run -- -a chromecast.local -s

The following app has been stopped: Default Media Receiver (CC1AD845)
```

### Media features
```bash
// Stream a video.
$ cargo run -- -a chromecast.local -m http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4

// Stream a video of specific format with buffering.
$ cargo run -- -a chromecast.local -m http://xxx.webm --media-type video/webm --media-stream-type buffered

// Stream video from YouTube.
$ cargo run -- -a chromecast.local -m 7LcUOEP7Brc --media-app youtube

// Display an image.
$ cargo run -- -a chromecast.local -m https://azasypkin.github.io/style-my-image/images/mozilla.jpg
```

For all possible values of `--media-type` see [Supported Media for Google Cast](https://developers.google.com/cast/docs/media).