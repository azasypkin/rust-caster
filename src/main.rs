#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate chromecast_link;

use chromecast_link::Chromecast;
use chromecast_link::CHROMECAST_APPS;
use chromecast_link::channels::receiver;
use chromecast_link::channels::media::StreamType;

const DEFAULT_DESTINATION_ID: &'static str = "receiver-0";

docopt!(Args derive Debug, "
Usage: chromecast-link-tool [-v] [-h] [-a <address>] [-p <port>] [--stream <url>] [--media-type <media type>] [--stream-type <stream type>]

Options:
    -a, --address <address>             Chromecast's network address.
    -p, --port <port>                   Chromecast's network port. [default: 8009]
        --stream <url>                  Stream specified URL to Chromecast connected device.
        --media-type <media_type>       Media type of the video to play. [default: video/mp4]
        --stream-type <stream_type>     Stream type to use (buffered, live or unknown). [default: unknown]
    -v, --verbose                       Toggle verbose output.
    -h, --help                          Print this help menu.
",
        flag_address: Option<String>,
        flag_port: u16,
        flag_stream: Option<String>,
        flag_media_type: String,
        flag_stream_type: String
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

    let mut media_loaded = false;
    let stream_url: String = args.flag_stream.unwrap_or("".to_owned());
    let media_type = args.flag_media_type;

    chromecast.connection.connect(DEFAULT_DESTINATION_ID.to_owned());
    chromecast.heartbeat.ping();
    chromecast.receiver.launch_app(CHROMECAST_APPS.default_media_receiver.to_owned());

    loop {
        let message = chromecast.receive().unwrap();

        if let Ok(payload) = chromecast.heartbeat.try_handle(&message) {
            if payload.typ == "PING" {
                chromecast.heartbeat.pong();
            }
        } else if let Ok(payload) = chromecast.connection.try_handle(&message) {
            println!("Connection channel message received: {:?}", payload);
        } else if let Ok(payload) = chromecast.receiver.try_handle(&message) {
            match payload {
                receiver::Reply::Status(reply) => {
                    if reply.status.applications.len() > 0 && !media_loaded && !stream_url.is_empty() {
                        let application = &reply.status.applications[0];

                        chromecast.connection.connect(application.transport_id.clone());

                        let media_channel = chromecast.create_media_channel(
                            application.transport_id.clone(),
                            application.session_id.clone()).unwrap();


                        let stream_type = match args.flag_stream_type.as_ref() {
                            "buffered" => StreamType::Buffered,
                            "live" => StreamType::Live,
                            "unknown" => StreamType::Unknown,
                            _ => panic!(
                                "Unsupported stream type {}! Valid values are: buffered, live or unknown. ",
                                args.flag_stream_type)
                        };

                        media_channel.stream(stream_url.as_ref(), media_type.as_ref(), stream_type);

                        media_loaded = true;
                    }
                }
                _ => {
                    println!("Receiver channel message received: {:?}", payload);
                }
            }
        }
    }
}
