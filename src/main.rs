extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate rust_cast;

use std::str::FromStr;

use ansi_term::Colour::{Green, Red};

use docopt::Docopt;

use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::connection::ConnectionResponse;
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::media::{StreamType, MediaResponse};
use rust_cast::channels::receiver::{CastDeviceApp, ReceiverResponse, Status as ReceiverStatus};

const DEFAULT_DESTINATION_ID: &'static str = "receiver-0";

const USAGE: &'static str = "
Usage: rust-caster [-v] [-h] [-a <address>] [-p <port>] [-r <app to run>] [-s] [-i] [-m <media handle>] [--media-type <media type>] [--video-stream-type <stream type>] [--media-app <media app>] [--media-volume <level> | --media-mute| --media-unmute | --media-pause | --media-play | --media-stop | --media-seek <time>]

Options:
    -a, --address <address>                 Cast device network address.
    -p, --port <port>                       Cast device network port. [default: 8009]
    -r, --run <app_to_run>                  Run the app with specified id/name.
    -s, --stop                              Stops currently active app.
    -i, --info                              Returns the info about the receiver.
    -m, --media <media_handle>              Media handle (URL for image or video, URL token for youtube video etc.) to load on the Cast connected device.
        --media-type <media_type>           Type of the media to load.
        --media-app <media_app>             Media app to use for streaming. [default: default]
        --media-stream-type <stream_type>   Media stream type to use (buffered, live or none). [default: none]
        --media-volume <level>              Media volume level.
        --media-mute                        Mute cast device.
        --media-unmute                      Unmute cast device.
        --media-pause                       Pause currently active media.
        --media-play                        Play currently paused media.
        --media-stop                        Stops currently active media.
        --media-seek <time>                 Sets the current position in the media stream.
    -v, --verbose                           Toggle verbose output.
    -h, --help                              Print this help menu.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_address: Option<String>,
    flag_port: u16,
    flag_run: Option<String>,
    flag_stop: Option<String>,
    flag_info: Option<String>,
    flag_media: Option<String>,
    flag_media_type: Option<String>,
    flag_media_app: String,
    flag_media_stream_type: String,
    flag_media_volume: Option<f32>,
    flag_media_mute: bool,
    flag_media_unmute: bool,
    flag_media_pause: bool,
    flag_media_play: bool,
    flag_media_stop: bool,
    flag_media_seek: Option<f32>,
}

fn main() {
    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_address.is_none() {
        println!("Please specify Cast Device address!");
        std::process::exit(1);
    }

    let cast_device = match CastDevice::connect(args.flag_address.unwrap(), args.flag_port) {
        Ok(cast_device) => cast_device,
        Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err)
    };

    cast_device.connection.connect(DEFAULT_DESTINATION_ID.to_owned()).unwrap();
    cast_device.heartbeat.ping().unwrap();
    cast_device.receiver.get_status().unwrap();

    let media = args.flag_media.unwrap_or("".to_owned());
    let media_type = args.flag_media_type.unwrap_or("".to_owned());

    let mut current_status: Option<ReceiverStatus> = None;

    loop {
        match cast_device.receive() {
            Ok(ChannelMessage::Connection(response)) => {
                match response {
                    ConnectionResponse::Connect => debug!("[Connection] Connect message received."),
                    ConnectionResponse::Close => debug!("[Connection] Close message received."),
                    ConnectionResponse::NotImplemented(typ, value) => {
                        warn!("[Connection] Support for the following message type `{}` is not yet
                               implemented {:?}", typ, value);
                    }
                };
            },

            Ok(ChannelMessage::Heartbeat(response)) => {
                match response {
                    HeartbeatResponse::Ping => {
                        debug!("[Heartbeat] Ping message received.");
                        cast_device.heartbeat.pong().unwrap();
                    },
                    HeartbeatResponse::Pong => debug!("[Heartbeat] Pong message received."),
                    HeartbeatResponse::NotImplemented(typ, value) => {
                        warn!("[Heartbeat] Support for the following message type `{}` is not yet
                               implemented {:?}", typ, value);
                    }
                };
            },

            Ok(ChannelMessage::Media(response)) => {
                match response {
                    MediaResponse::Status(statuses) => {
                        debug!("[Media] Status message received {:?}.", statuses);

                        if !media.is_empty() {
                            let current_media = statuses.iter().find(|ref status| {
                                if let Some(ref current_media) = status.media {
                                    return current_media.content_id == media;
                                }

                                false
                            });

                            if current_media.is_some() {
                                break;
                            }
                        } else if args.flag_media_pause || args.flag_media_play ||
                            args.flag_media_stop || args.flag_media_seek.is_some() {
                            match statuses.first() {
                                None => {},
                                Some(status) => {
                                    let media_app = CastDeviceApp::from_str(
                                        args.flag_media_app.as_ref()).unwrap();

                                    let ref apps = current_status.as_ref().unwrap().applications;

                                    let app = apps.iter().find(|ref app| {
                                        CastDeviceApp::from_str(app.app_id.as_ref()).unwrap() ==
                                            media_app
                                    });

                                    match app {
                                        None => break,
                                        Some(app) => {
                                            if args.flag_media_pause {
                                                cast_device.media.pause(
                                                    app.transport_id.as_ref(),
                                                    status.media_session_id).unwrap();
                                            } else if args.flag_media_play {
                                                cast_device.media.play(
                                                    app.transport_id.as_ref(),
                                                    status.media_session_id).unwrap();
                                            } else if args.flag_media_stop {
                                                cast_device.media.stop(
                                                    app.transport_id.as_ref(),
                                                    status.media_session_id).unwrap();
                                            } else if args.flag_media_seek.is_some() {
                                                cast_device.media.seek(
                                                    app.transport_id.as_ref(),
                                                    status.media_session_id,
                                                    Some(args.flag_media_seek.unwrap()),
                                                    None).unwrap();
                                            }
                                        }
                                    }


                                    break;
                                }
                            }
                        }

                    },
                    MediaResponse::LoadCancelled => {
                        debug!("[Media] Load cancelled message received.");
                    },
                    MediaResponse::NotImplemented(typ, value) => {
                        warn!("[Media] Support for the following message type `{}` is not yet
                               implemented {:?}", typ, value);
                    }
                }
            },

            Ok(ChannelMessage::Receiver(response)) => {
                match response {
                    ReceiverResponse::Status(status) => {
                        debug!("[Receiver] Status message received {:?}.", status);

                        current_status = Some(status);

                        let status = current_status.as_ref().unwrap();
                        let ref apps = status.applications;

                        if args.flag_info.is_some() {
                            println!("\n{} {}",
                                     Green.paint("Number of apps run:"),
                                     Red.paint(apps.len().to_string()));
                            for i in 0..apps.len() {
                                println!("{}{}{}{}{}{}{}",
                                         Green.paint("App#"),
                                         Green.paint(i.to_string()),
                                         Green.paint(": "),
                                         Red.paint(apps[i].display_name.as_ref()),
                                         Red.paint(" ("),
                                         Red.paint(apps[i].app_id.as_ref()),
                                         Red.paint(")"));
                            }

                            if status.volume.level.is_some() {
                                println!("{} {}",
                                         Green.paint("Volume level:"),
                                         Red.paint(status.volume.level.unwrap().to_string()));
                            }

                            if status.volume.muted.is_some() {
                                println!("{} {}\n",
                                         Green.paint("Muted:"),
                                         Red.paint(status.volume.muted.unwrap().to_string()));
                            }
                            break;
                        } else if args.flag_run.is_some() {
                            let app = CastDeviceApp::from_str(
                                args.flag_run.as_ref().unwrap()).unwrap();
                            cast_device.receiver.launch_app(app).unwrap();
                            break;
                        } else if args.flag_stop.is_some() {
                            if apps.len() == 0 {
                                println!("{}", Red.paint("There is no app to stop!"));
                            } else {
                                cast_device.receiver.stop_app(apps[0].session_id.as_ref()).unwrap();
                                println!("{}{}{}{}{}",
                                         Green.paint("The following app has been stopped: "),
                                         Red.paint(apps[0].display_name.as_ref()),
                                         Red.paint(" ("),
                                         Red.paint(apps[0].app_id.as_ref()),
                                         Red.paint(")"));
                            }

                            break;
                        } else if !media.is_empty() {
                            // Check if required app is run.
                            let media_app = CastDeviceApp::from_str(
                                args.flag_media_app.as_ref()).unwrap();

                            let app = apps.iter().find(|ref app| {
                                CastDeviceApp::from_str(app.app_id.as_ref()).unwrap() == media_app
                            });

                            match app {
                                None => cast_device.receiver.launch_app(media_app).unwrap(),
                                Some(app) => {
                                    cast_device.connection.connect(
                                        app.transport_id.as_ref()).unwrap();

                                    let media_stream_type = match args.flag_media_stream_type.as_ref() {
                                        "buffered" => StreamType::Buffered,
                                        "live" => StreamType::Live,
                                        "none" => StreamType::None,
                                        _ => panic!("Unsupported stream type {}!",
                                                args.flag_media_stream_type)
                                    };

                                    cast_device.media.load(app.transport_id.as_ref(),
                                                          app.session_id.as_ref(), media.as_ref(),
                                                          media_type.as_ref(),
                                                          media_stream_type).unwrap();
                                }
                            }
                        } else if args.flag_media_volume.is_some() {
                            let level = args.flag_media_volume.unwrap();
                            cast_device.receiver.set_volume(level).unwrap();
                            println!("{}{}", Green.paint("Volume level has been set to: "),
                                     Red.paint(level.to_string()));

                            break;
                        } else if args.flag_media_mute {
                            cast_device.receiver.set_volume(true).unwrap();
                            println!("{}", Green.paint("Cast device is muted."));
                            break;
                        } else if args.flag_media_unmute {
                            cast_device.receiver.set_volume(false).unwrap();
                            println!("{}", Green.paint("Cast device is unmuted."));
                            break;
                        } else if args.flag_media_pause || args.flag_media_play ||
                            args.flag_media_stop || args.flag_media_seek.is_some() {

                            let media_app = CastDeviceApp::from_str(
                                args.flag_media_app.as_ref()).unwrap();

                            let app = apps.iter().find(|ref app| {
                                CastDeviceApp::from_str(app.app_id.as_ref()).unwrap() == media_app
                            });

                            match app {
                                None => break,
                                Some(app) => {
                                    cast_device.connection.connect(
                                        app.transport_id.as_ref()).unwrap();
                                    cast_device.media.get_status(
                                        app.transport_id.as_ref()).unwrap();
                                }
                            }
                        }
                    },
                    ReceiverResponse::LaunchError => {
                        debug!("[Receiver] Launch error message received.");
                    },
                    ReceiverResponse::NotImplemented(typ, value) => {
                        warn!("[Receiver] Support for the following message type `{}` is not yet
                               implemented {:?}", typ, value);
                    }
                }
            },

            Ok(ChannelMessage::Raw(response)) => {
                debug!("Support for the following message type is not yet supported: {:?}",
                       response);
            }

            Err(error) => error!("Error occurred while receiving message {}", error)
        }
    }
}
