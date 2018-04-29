extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rust_cast;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate regex;

use std::str::FromStr;

use ansi_term::Colour::{Green, Red};

use docopt::Docopt;

use rust_cast::{CastDevice, ChannelMessage};
use rust_cast::channels::heartbeat::HeartbeatResponse;
use rust_cast::channels::media::{Media, StatusEntry, StreamType};
use rust_cast::channels::receiver::CastDeviceApp;
use std::process::Command;

const DEFAULT_DESTINATION_ID: &str = "receiver-0";

const USAGE: &str = "
Usage: rust-caster [-v] [-h] [-a <address>] [-p <port>] [-i | -r <app to run> | -s <app to stop> | --stop-current | [-m <media handle> [--media-type <media type>] [--media-stream-type <stream type>] [--media-app <media app>]] | [--media-volume <level> | --media-mute| --media-unmute | --media-pause | --media-play | --media-stop | --media-seek <time>] [--media-app <media app>]]

Options:
    -a, --address <address>                 Cast device network address.
    -p, --port <port>                       Cast device network port. [default: 8009]
    -r, --run <app_to_run>                  Run the app with specified id/name.
    -s, --stop <app_to_stop>                Stops the app with specified id/name.
        --stop-current                      Stops currently active app.
    -i, --info                              Returns the info about the receiver.
    -m, --media <media_handle>              Media handle (URL for image or video, URL token for youtube video etc.) to load on the Cast connected device.
        --media-type <media_type>           Type of the media to load.
        --media-app <media_app>             Media app to use for streaming. [default: default]
        --media-stream-type <stream_type>   Media stream type to use (buffered, live or none). [default: none]
        --media-volume <level>              Media volume level.
        --media-mute                        Mute cast device.
        --media-unmute                      Unmute cast device.
        --media-pause                       Pause currently active media in the app that is passed in `--media-app`.
        --media-play                        Play currently paused media in the app that is passed in `--media-app`.
        --media-stop                        Stops currently active media in the app that is passed in `--media-app`.
        --media-seek <time>                 Sets the current position in the media stream in the app that is passed in `--media-app`.
    -v, --verbose                           Toggle verbose output.
    -h, --help                              Print this help menu.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_address: Option<String>,
    flag_port: u16,
    flag_run: Option<String>,
    flag_stop: Option<String>,
    flag_stop_current: bool,
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

fn print_info(device: &CastDevice) {
    let status = device.receiver.get_status().unwrap();

    println!(
        "\n{} {}",
        Green.paint("Number of apps run:"),
        Red.paint(status.applications.len().to_string())
    );
    for i in 0..status.applications.len() {
        println!(
            "{}{}{}{}{}{}{}{}{}",
            Green.paint("App#"),
            Green.paint(i.to_string()),
            Green.paint(": "),
            Red.paint(status.applications[i].display_name.as_ref()),
            Red.paint(" ("),
            Red.paint(status.applications[i].app_id.as_ref()),
            Red.paint(")"),
            Red.paint(" - "),
            Red.paint(status.applications[i].status_text.as_ref())
        );
    }

    if let Some(level) = status.volume.level {
        println!(
            "{} {}",
            Green.paint("Volume level:"),
            Red.paint(level.to_string())
        );
    }

    if let Some(muted) = status.volume.muted {
        println!(
            "{} {}\n",
            Green.paint("Muted:"),
            Red.paint(muted.to_string())
        );
    }
}

fn run_app(device: &CastDevice, app_to_run: &CastDeviceApp) {
    let app = device.receiver.launch_app(app_to_run).unwrap();

    println!(
        "{}{}{}{}{}{}{}",
        Green.paint("The following application has been run: "),
        Red.paint(app.display_name),
        Red.paint(" ("),
        Red.paint(app.app_id),
        Red.paint(")"),
        Red.paint(" - "),
        Red.paint(app.status_text)
    );
}

fn stop_app(device: &CastDevice, app_to_run: &CastDeviceApp) {
    let status = device.receiver.get_status().unwrap();

    let app = status
        .applications
        .iter()
        .find(|app| &CastDeviceApp::from_str(app.app_id.as_ref()).unwrap() == app_to_run);

    match app {
        Some(app) => {
            device.receiver.stop_app(app.session_id.as_ref()).unwrap();

            println!(
                "{}{}{}{}{}{}{}",
                Green.paint("The following application has been stopped: "),
                Red.paint(app.display_name.as_ref()),
                Red.paint(" ("),
                Red.paint(app.app_id.as_ref()),
                Red.paint(")"),
                Red.paint(" - "),
                Red.paint(app.status_text.as_ref())
            );
        }
        None => {
            println!(
                "{} `{}` {}",
                Green.paint("Application"),
                Red.paint(app_to_run.to_string()),
                Green.paint("is not run!")
            );
        }
    }
}

fn stop_current_app(device: &CastDevice) {
    let status = device.receiver.get_status().unwrap();
    match status.applications.first() {
        Some(app) => {
            device.receiver.stop_app(app.session_id.as_ref()).unwrap();

            println!(
                "{}{}{}{}{}{}{}",
                Green.paint("The following application has been stopped: "),
                Red.paint(app.display_name.as_ref()),
                Red.paint(" ("),
                Red.paint(app.app_id.as_ref()),
                Red.paint(")"),
                Red.paint(" - "),
                Red.paint(app.status_text.as_ref())
            );
        }
        None => println!("{}", Green.paint("There are no applications active!")),
    }
}

fn play_media(device: &CastDevice, app_to_run: &CastDeviceApp, media: String, media_type: String, media_stream_type: StreamType) {
    let app = device.receiver.launch_app(app_to_run).unwrap();

    device
        .connection
        .connect(app.transport_id.as_ref())
        .unwrap();

    let status = device
        .media
        .load(
            app.transport_id.as_ref(),
            app.session_id.as_ref(),
            &Media {
                content_id: media,
                content_type: media_type,
                stream_type: media_stream_type,
                duration: None,
                metadata: None,
            },
        )
        .unwrap();

    for i in 0..status.entries.len() {
        println!(
            "{}{}{}",
            Green.paint("Media#"),
            Green.paint(i.to_string()),
            Green.paint(": ")
        );
        println!(
            "{} {}",
            Green.paint("Playback rate:"),
            Red.paint(status.entries[i].playback_rate.to_string())
        );
        println!(
            "{} {}",
            Green.paint("Player state:"),
            Red.paint(status.entries[i].player_state.to_string())
        );

        if let Some(time) = status.entries[i].current_time {
            println!(
                "{} {}",
                Green.paint("Current time:"),
                Red.paint(time.to_string())
            );
        }

        if let Some(ref media) = status.entries[i].media {
            println!(
                "{} {}",
                Green.paint("Content Id:"),
                Red.paint(media.content_id.as_ref())
            );
            println!(
                "{} {}",
                Green.paint("Stream type:"),
                Red.paint(media.stream_type.to_string())
            );
            println!(
                "{} {}",
                Green.paint("Content type:"),
                Red.paint(media.content_type.as_ref())
            );

            if let Some(duration) = media.duration {
                println!(
                    "{} {}",
                    Green.paint("Duration:"),
                    Red.paint(duration.to_string())
                );
            }
        }
    }
}

fn yes_no(question: &'static str) -> bool {
    println!("{}", question);

    loop {
        println!("Answer (y/n):");
	let mut response = String::new();
        std::io::stdin().read_line(&mut response).expect("Does not understand input");
        response = response.trim().to_string();

	match response.as_ref() {
	      "y" => return true,
	      "n" => return false,
	      _ => println!("Unknown response!")
	};
    }
}

fn find_chromecast_device() -> Result<String, &'static str> {

    let mut ip_addresses: Vec<String> = Vec::new();

    let output = Command::new("sh")
        .arg("-c")
	.arg("avahi-browse -a --resolve -t")
        .output()
        .expect("failed to execute avahi-browse");
    let output = String::from_utf8_lossy(&output.stdout);
    let re = regex::Regex::new(r"(?s)=[\d\w\W\s]+Chromecast-.*?(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
    for cap in re.captures_iter(&output) {
	ip_addresses.push(String::from(&cap[1]));
    }

    if ip_addresses.len() == 0 {
        return Err("Did not find any Cast Devices");
    }

    if ip_addresses.len() > 1 {
        println!("Chromecast found on the following IP addresses:");
        
        for (i, ip) in ip_addresses.iter().enumerate() {
            println!("{}: {}", i, &ip);
        }
        
        let high_index = ip_addresses.len();
        let mut selected_index = high_index + 1;
        while selected_index >= high_index {
   	    println!("Please select IP address by index:");
	    let mut input = String::new();
	    std::io::stdin().read_line(&mut input).expect("Failed to read line");
	    selected_index = input.trim().parse().expect("Please type a valid number!");
        }

        return Ok(ip_addresses.get(selected_index).unwrap().clone());
    }

    return Ok(ip_addresses.get(0).unwrap().clone());
}

fn main() {
    env_logger::init();

    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_address.is_none() {
        if cfg!(target_os = "linux") && yes_no("Did not provide Cast Device address. Do you want to search your network?") {
            
            match find_chromecast_device(){
                Ok(addr) => {
                    println!("Found Casting Device on {}", addr);
                    args.flag_address = Some(addr);
                },
                Err(e) =>  {
                    println!("{}", e);
                    std::process::exit(1);   
                }
            }
        } else {
            println!("Please specify Cast Device address!");
            std::process::exit(1);
        }
    }

    let cast_device = match CastDevice::connect_without_host_verification(args.flag_address.unwrap(), args.flag_port) {
        Ok(cast_device) => cast_device,
        Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
    };

    cast_device
        .connection
        .connect(DEFAULT_DESTINATION_ID.to_string())
        .unwrap();
    cast_device.heartbeat.ping().unwrap();

    // Information about cast device.
    if args.flag_info.is_some() {
        return print_info(&cast_device);
    }

    // Run specific application.
    if let Some(app) = args.flag_run {
        return run_app(&cast_device, &CastDeviceApp::from_str(&app).unwrap());
    }

    // Stop specific application.
    if let Some(app) = args.flag_stop {
        return stop_app(&cast_device, &CastDeviceApp::from_str(&app).unwrap());
    }

    // Stop currently active application.
    if args.flag_stop_current {
        return stop_current_app(&cast_device);
    }

    // Adjust volume level.
    if let Some(level) = args.flag_media_volume {
        let volume = cast_device.receiver.set_volume(level).unwrap();
        println!(
            "{}{}",
            Green.paint("Volume level has been set to: "),
            Red.paint(volume.level.unwrap_or(level).to_string())
        );
        return;
    }

    // Mute/unmute cast device.
    if args.flag_media_mute || args.flag_media_unmute {
        let mute_or_unmute = args.flag_media_mute;
        let volume = cast_device.receiver.set_volume(mute_or_unmute).unwrap();
        println!(
            "{}{}",
            Green.paint("Cast device is muted: "),
            Red.paint(volume.muted.unwrap_or(mute_or_unmute).to_string())
        );
        return;
    }

    // Manage media session playback (play, pause, stop and seek).
    if args.flag_media_pause || args.flag_media_play || args.flag_media_stop || args.flag_media_seek.is_some() {
        let app_to_manage = CastDeviceApp::from_str(args.flag_media_app.as_ref()).unwrap();
        let status = cast_device.receiver.get_status().unwrap();

        let app = status
            .applications
            .iter()
            .find(|app| CastDeviceApp::from_str(app.app_id.as_ref()).unwrap() == app_to_manage);

        match app {
            Some(app) => {
                cast_device
                    .connection
                    .connect(app.transport_id.as_ref())
                    .unwrap();

                let status = cast_device
                    .media
                    .get_status(app.transport_id.as_ref(), None)
                    .unwrap();
                let status = status.entries.first().unwrap();

                let mut status_entry: Option<StatusEntry> = None;

                if args.flag_media_pause {
                    status_entry = Some(
                        cast_device
                            .media
                            .pause(app.transport_id.as_ref(), status.media_session_id)
                            .unwrap(),
                    );
                } else if args.flag_media_play {
                    status_entry = Some(
                        cast_device
                            .media
                            .play(app.transport_id.as_ref(), status.media_session_id)
                            .unwrap(),
                    );
                } else if args.flag_media_stop {
                    status_entry = Some(
                        cast_device
                            .media
                            .stop(app.transport_id.as_ref(), status.media_session_id)
                            .unwrap(),
                    );
                } else if args.flag_media_seek.is_some() {
                    status_entry = Some(
                        cast_device
                            .media
                            .seek(
                                app.transport_id.as_ref(),
                                status.media_session_id,
                                Some(args.flag_media_seek.unwrap()),
                                None,
                            )
                            .unwrap(),
                    );
                }

                if let Some(status_entry) = status_entry {
                    println!("{}", Green.paint("Media:"));
                    println!(
                        "{} {}",
                        Green.paint("Playback rate:"),
                        Red.paint(status_entry.playback_rate.to_string())
                    );
                    println!(
                        "{} {}",
                        Green.paint("Player state:"),
                        Red.paint(status_entry.player_state.to_string())
                    );

                    if let Some(time) = status_entry.current_time {
                        println!(
                            "{} {}",
                            Green.paint("Current time:"),
                            Red.paint(time.to_string())
                        );
                    }

                    if let Some(ref media) = status_entry.media {
                        println!(
                            "{} {}",
                            Green.paint("Content Id:"),
                            Red.paint(media.content_id.as_ref())
                        );
                        println!(
                            "{} {}",
                            Green.paint("Stream type:"),
                            Red.paint(media.stream_type.to_string())
                        );
                        println!(
                            "{} {}",
                            Green.paint("Content type:"),
                            Red.paint(media.content_type.as_ref())
                        );

                        if let Some(duration) = media.duration {
                            println!(
                                "{} {}",
                                Green.paint("Duration:"),
                                Red.paint(duration.to_string())
                            );
                        }
                    }
                }
            }
            None => {
                println!(
                    "{} `{}` {}",
                    Green.paint("Application"),
                    Red.paint(app_to_manage.to_string()),
                    Green.paint("is not run!")
                );
            }
        }
        return;
    }

    // Play media and keep connection.
    if let Some(media) = args.flag_media {
        let media_type = args.flag_media_type.unwrap_or_else(|| "".to_string());

        let media_stream_type = match args.flag_media_stream_type.as_ref() {
            value @ "buffered" | value @ "live" | value @ "none" => StreamType::from_str(value).unwrap(),
            _ => panic!("Unsupported stream type {}!", args.flag_media_stream_type),
        };

        play_media(
            &cast_device,
            &CastDeviceApp::from_str(args.flag_media_app.as_ref()).unwrap(),
            media,
            media_type,
            media_stream_type,
        );

        loop {
            match cast_device.receive() {
                Ok(ChannelMessage::Heartbeat(response)) => {
                    println!("[Heartbeat] {:?}", response);

                    if let HeartbeatResponse::Ping = response {
                        cast_device.heartbeat.pong().unwrap();
                    }
                }

                Ok(ChannelMessage::Connection(response)) => println!("[Connection] {:?}", response),
                Ok(ChannelMessage::Media(response)) => println!("[Media] {:?}", response),
                Ok(ChannelMessage::Receiver(response)) => println!("[Receiver] {:?}", response),
                Ok(ChannelMessage::Raw(response)) => println!(
                    "Support for the following message type is not yet supported: {:?}",
                    response
                ),

                Err(error) => error!("Error occurred while receiving message {}", error),
            }
        }
    }
}
