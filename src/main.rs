extern crate chromecast_link;

use chromecast_link::Chromecast;
use chromecast_link::CHROMECAST_APPS;
use chromecast_link::channels::receiver;

const DEFAULT_DESTINATION_ID: &'static str = "receiver-0";

fn main() {
    let mut chromecast = Chromecast::new("az_chromecast.local".to_owned(), 8009);

    if let Err(err) = chromecast.connect() {
        panic!("Chromecast is unable to establish connection: {:?}", err);
    }

    let heartbeat_channel = chromecast.create_heartbeat_channel();
    let connection_channel = chromecast.create_connection_channel();
    let receiver_channel = chromecast.create_receiver_channel();

    let mut media_loaded = false;

    connection_channel.connect(DEFAULT_DESTINATION_ID.to_owned());
    heartbeat_channel.ping();
    receiver_channel.launch_app(CHROMECAST_APPS.default_media_receiver.to_owned());

    loop {
        let message = chromecast.receive();

        if let Ok(payload) = heartbeat_channel.try_handle(&message) {
            if payload.typ == "PING" {
                heartbeat_channel.pong();
            }
        } else if let Ok(payload) = connection_channel.try_handle(&message) {
            println!("Connection channel message received: {:?}", payload);
        } else if let Ok(payload) = receiver_channel.try_handle(&message) {
            match payload {
                receiver::Reply::Status(reply) => {
                    if reply.status.applications.len() > 0 && !media_loaded {
                        let application = &reply.status.applications[0];

                        // Connect to application.
                        connection_channel.connect(application.transport_id.clone());

                        let media_channel =
                            chromecast.create_media_channel(application.transport_id.clone(),
                                                            application.session_id.clone());

                        media_channel.load("http://commondatastorage.googleapis.\
                                            com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
                                               .to_owned(),
                                           "video/mp4".to_owned());

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
