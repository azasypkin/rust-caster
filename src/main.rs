#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate chromecast_link;

use std::str::FromStr;

use ansi_term::Colour::{Green, Red};

use chromecast_link::Chromecast;
use chromecast_link::channels::receiver::{ChromecastApp, Reply};
use chromecast_link::channels::media::StreamType;

const DEFAULT_DESTINATION_ID: &'static str = "receiver-0";

docopt!(Args derive Debug, "
Usage: chromecast-link-tool [-v] [-h] [-a <address>] [-p <port>] [-r <app to run>] [-s] [-i] [-m <media handle>] [--media-type <media type>] [--video-stream-type <stream type>] [--media-app <media app>]

Options:
    -a, --address <address>                 Chromecast's network address.
    -p, --port <port>                       Chromecast's network port. [default: 8009]
    -r, --run <app_to_run>                  Run the app with specified id/name.
    -s, --stop                              Stops currently active app.
    -i, --info                              Returns the info about the receiver.
    -m, --media <media_handle>              Media handle (URL for image or video, URL token for youtube video etc.) to load on the Chromecast connected device.
        --media-type <media_type>           Type of the media to load.
        --media-app <media_app>             Media app to use for streaming. [default: default]
        --media-stream-type <stream_type>   Media stream type to use (buffered, live or none). [default: none]
    -v, --verbose                           Toggle verbose output.
    -h, --help                              Print this help menu.
",
        flag_address: Option<String>,
        flag_port: u16,
        flag_run: Option<String>,
        flag_stop: Option<String>,
        flag_info: Option<String>,
        flag_media: Option<String>,
        flag_media_type: Option<String>,
        flag_media_app: String,
        flag_media_stream_type: String,
);

fn main() {
    env_logger::init().unwrap();

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    if args.flag_address.is_none() {
        println!("Please specify Chromecast's address!");
        std::process::exit(1);
    }

    let chromecast = match Chromecast::connect(args.flag_address.unwrap(), args.flag_port) {
        Ok(chromecast) => chromecast,
        Err(err) => panic!("Chromecast is unable to establish connection: {:?}", err)
    };

    chromecast.connection.connect(DEFAULT_DESTINATION_ID.to_owned()).unwrap();
    chromecast.heartbeat.ping().unwrap();
    chromecast.receiver.get_status().unwrap();

    let media = args.flag_media.unwrap_or("".to_owned());
    let media_type = args.flag_media_type.unwrap_or("".to_owned());

    loop {
        let message = chromecast.receive().unwrap();

        if let Ok(payload) = chromecast.heartbeat.try_handle(&message) {
            if payload.typ == "PING" {
                chromecast.heartbeat.pong().unwrap();
            }
        } else if let Ok(payload) = chromecast.connection.try_handle(&message) {
            debug!("Connection channel message received: {:?}", payload);
        } else if let Ok(payload) = chromecast.receiver.try_handle(&message) {
            match payload {
                Reply::Status(reply) => {
                    let apps = reply.status.applications;

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
                        println!("{} {}",
                                 Green.paint("Volume level:"),
                                 Red.paint(reply.status.volume.level.to_string()));
                        println!("{} {}\n",
                                 Green.paint("Muted:"),
                                 Red.paint(reply.status.volume.muted.to_string()));
                        break;
                    } else if args.flag_run.is_some() {
                        let app = ChromecastApp::from_str(args.flag_run.as_ref().unwrap()).unwrap();
                        chromecast.receiver.launch_app(app).unwrap();
                      break;
                    } else if args.flag_stop.is_some() {
                        if apps.len() == 0 {
                            println!("{}", Red.paint("There is no app to stop!"));
                        } else {
                            chromecast.receiver.stop_app(apps[0].session_id.as_ref()).unwrap();
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
                        let media_app = ChromecastApp::from_str(
                            args.flag_media_app.as_ref()).unwrap();

                        let app = apps.iter().find(|ref app| {
                            ChromecastApp::from_str(app.app_id.as_ref()).unwrap() == media_app
                        });

                        match app {
                            None => chromecast.receiver.launch_app(media_app).unwrap(),
                            Some(app) => {
                                chromecast.connection.connect(app.transport_id.as_ref()).unwrap();

                                let media_channel = chromecast.create_media_channel(
                                    app.transport_id.as_ref(),
                                    app.session_id.as_ref()).unwrap();

                                let media_stream_type = match args.flag_media_stream_type.as_ref() {
                                    "buffered" => StreamType::Buffered,
                                    "live" => StreamType::Live,
                                    "none" => StreamType::None,
                                    _ => panic!("Unsupported stream type {}!",
                                                args.flag_media_stream_type)
                                };

                                media_channel.load(media.as_ref(), media_type.as_ref(),
                                                   media_stream_type).unwrap();
                            }
                        }
                    }
                }
                _ => {
                    println!("Receiver channel message received: {:?}", payload);
                }
            }
        }
    }
}
