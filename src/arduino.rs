//#![windows_subsystem = "windows"]
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use rouille::websocket::Message;
use serde::Serialize;
use crate::{CommandOutput, GENERIC_OK};

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const ARDUINO_PORT:i32 = 8282;
const SKETCHES_FOLDER:&str = "./sketches";
const ARDUINO_VERSION:&str = "1.0.2";

#[derive(Serialize)]
struct CompileResponse {
    success:bool,
    used_bytes:i32,
    used_percent:i32,
    max_bytes:i32,
    message:String,
}

#[derive(Serialize)]
struct UploadResponse {
    success:bool,
    port:String,
    message:String,
}

//2.0: use json from cli

pub fn start_arduino() {
    run_cli_command(vec!["core","update-index"]);
    fs::create_dir_all(SKETCHES_FOLDER).unwrap();

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{ARDUINO_PORT}"), move |request| {
        router!(request,
            (POST) (/create/{name:String}) => {
                run_cli_command(vec![
                    "sketch",
                    "new",
                    name.as_str(),
                ]);

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/upload/{name:String}) => {

                 let board_out = run_cli_command(vec![
                     "board",
                     "list",
                 ]);
                //let board_out = "Port Protocol Type          Board Name FQBN            Core\nCOM1 serial   Serial Port   Unknown\nCOM4 serial   Serial Port (USB) Unknown".to_string();
                println!("{}", board_out);

                let board_out_words = board_out.split_whitespace().collect::<Vec<&str>>();
                if(board_out_words.len()==3){
                    return rouille::Response::json(&UploadResponse{
                        success:false,
                        port:"".to_string(),
                        message: board_out,
                    }).with_additional_header("Access-Control-Allow-Origin", "*");
                }

                let margin = 7;
                let mut port="COM3";
                let mut i=0;
                loop{
                    if margin+5*i >= board_out_words.len(){
                        break;
                    }
                    if margin+4+5*i >= board_out_words.len(){
                        break;
                    }
                    let get_port = board_out_words[margin+5*i];
                    let is_usb = board_out_words.get(margin+4+5*i).unwrap().eq(&"(USB)".to_string());
                    port = get_port.clone();
                    if is_usb{
                        break;
                    }
                    i+=1;
                }

                let cmd_out = run_cli_command_collect_all(vec![
                    "upload",
                    "-p",
                    port,
                    "--fqbn",
                    "arduino:avr:nano",
                    name.as_str(),
                ]);
                let upload_out = cmd_out.stdout;

                rouille::Response::json(&UploadResponse{
                    success: true,
                    port:port.to_string(),
                    message: cmd_out.stderr,
                }).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/compile/{name:String}) => {
                let cmd_out =run_cli_command_collect_all(vec![
                    "compile",
                    "-b",
                    "arduino:avr:nano",
                    "-e",
                    name.as_str(),
                ]);
                let output = cmd_out.stdout;
                println!("/{}", output);
                println!("/{:?}", cmd_out.stderr);
                 let words = output.split(" ").collect::<Vec<&str>>();

                if !words[0].contains("Sketch"){
                    return rouille::Response::json(&CompileResponse{
                        success:false,
                        used_bytes:0,
                        used_percent:0,
                        max_bytes:0,
                        message: cmd_out.stderr.to_string().replace("\r","<br>"),
                    }).with_additional_header("Access-Control-Allow-Origin", "*");
                }


                let used_bytes = words[2].parse::<i32>().unwrap();
                let percent_str = words[4];
                let mut chars = percent_str.chars();
                chars.next();
                chars.next_back();
                chars.next_back();
                let used_percent = chars.as_str().parse::<i32>().unwrap();
                let max_bytes = words[11].parse::<i32>().unwrap();

                rouille::Response::json(&CompileResponse{
                    success:true,
                    used_bytes,
                    used_percent,
                    max_bytes,
                    message: output,
                }).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/write/{name:String}) => {
                let path =  format!("{SKETCHES_FOLDER}/{name}/{name}.ino");
                let mut buffer = String::new();
                let mut file = File::create(&path).unwrap();
                request.data().unwrap().read_to_string(&mut buffer).unwrap();
                file.write_all(buffer.as_bytes()).unwrap();

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/serial/{name:String}) => {
                let (mut socket, response) = connect(
                    Url::parse("wss://data.alpaca.markets/stream").unwrap()
                ).expect("Can't connect");

                socket.write_message(Message::Text(r#"{
                    "action": "authenticate",
                    "data": {
                        "key_id": "API-KEY",
                        "secret_key": "SECRET-KEY"
                    }
                }"#.into()));

                socket.write_message(Message::Text(r#"{
                    "action": "listen",
                    "data": {
                        "streams": ["AM.SPY"]
                    }
                }"#.into()));

                loop {
                    let msg = socket.read_message().expect("Error reading message");
                    println!("Received: {}", msg);
                }

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/status) => {
                rouille::Response::text(" ").with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/version) => {
                rouille::Response::text(ARDUINO_VERSION).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            _ => rouille::Response::empty_404()
        )
    });
}

fn run_cli_command(args:Vec<&str>)->String{
    run_cli_command_with_path(args,"").stdout
}

fn run_cli_command_collect_all(args:Vec<&str>)->CommandOutput{
    run_cli_command_with_path(args,"")
}

fn run_cli_command_with_path(args:Vec<&str>,dir:&str)->CommandOutput{
    let mut str = String::new();
    let mut binding = Command::new("arduino-cli")
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(format!("{SKETCHES_FOLDER}/{dir}"))
        .arg("--no-color")
        .args(args)
        .output()
        .unwrap();
    let stdout = binding
        .stdout
        .as_mut_slice();
    let stderr = binding
        .stderr
        .as_mut_slice();

    CommandOutput {
        stdout:std::str::from_utf8(&stdout).unwrap().to_string(),
        stderr: std::str::from_utf8(&stderr).unwrap().to_string(),
    }
}