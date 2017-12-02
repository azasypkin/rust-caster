# Rust Caster

[![Build Status](https://travis-ci.org/azasypkin/rust-caster.svg?branch=master)](https://travis-ci.org/azasypkin/rust-caster)

Just a helper tool for [Rust Cast](https://github.com/azasypkin/rust-cast) crate.

## Usage

### Generic features
To get the address of the device you can use `avahi` with the following command:
```bash
$ avahi-browse -a --resolve
```

```bash
// Get some info about the Google Cast enabled device (e.g. Chromecast). 
$ cargo run -- -a 192.168.0.100 -i

Number of apps run: 1
App#0: Default Media Receiver (CC1AD845)
Volume level: 1
Muted: false

// Run specific app on the Chromecast.
$ cargo run -- -a 192.168.0.100 -r youtube

// Stop specific active app.
$ cargo run -- -a 192.168.0.100 -s youtube

// Stop currently active app.
$ cargo run -- -a 192.168.0.100 --stop-current

The following app has been stopped: Default Media Receiver (CC1AD845)
```

### Media features
```bash
// Stream a video.
$ cargo run -- -a 192.168.0.100 -m http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4

// Stream a video of specific format with buffering.
$ cargo run -- -a 192.168.0.100 -m http://xxx.webm --media-type video/webm --media-stream-type buffered

// Stream video from YouTube.
$ cargo run -- -a 192.168.0.100 -m 7LcUOEP7Brc --media-app youtube

// Display an image.
$ cargo run -- -a 192.168.0.100 -m https://azasypkin.github.io/style-my-image/images/mozilla.jpg

// Change volume level.
$ cargo run -- -a 192.168.0.100 --media-volume 0.5

// Mute/unmute media.
$ cargo run -- -a 192.168.0.100 --media-mute [--media-unmute]

// Pause media.
$ cargo run -- -a 192.168.0.100 --media-app youtube --media-pause

// Resume/play media.
$ cargo run -- -a 192.168.0.100 --media-app youtube --media-play

// Seek media.
$ cargo run -- -a 192.168.0.100 --media-app youtube --media-seek 100
```

For all possible values of `--media-type` see [Supported Media for Google Cast](https://developers.google.com/cast/docs/media).