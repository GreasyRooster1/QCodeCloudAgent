//#![windows_subsystem = "windows"]
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use serde::Serialize;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const PORT:i32 = 8181;
const SKETCHES_FOLDER:&str = "./sketches";
const VERSION:&str = "1.0.2";

const GENERIC_OK:GenericResponse = GenericResponse{
    success:true
};

#[derive(Serialize)]
struct GenericResponse {
    success:bool
}

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

fn main() {
    run_cli_command(vec!["core","update-index"]);

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
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

                let upload_out = run_cli_command(vec![
                    "upload",
                    "-p",
                    port,
                    "--fqbn",
                    "arduino:avr:nano",
                    name.as_str(),
                ]);

                rouille::Response::json(&UploadResponse{
                    success: true,
                    port:port.to_string(),
                    message: upload_out,
                }).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/compile/{name:String}) => {
                let output = run_cli_command(vec![
                    "compile",
                    "-b",
                    "arduino:avr:nano",
                    "-e",
                    name.as_str(),
                ]);
                println!("/{}", output);
                 let words = output.split(" ").collect::<Vec<&str>>();

                if !words[0].contains("Sketch"){
                    return rouille::Response::json(&CompileResponse{
                        success:false,
                        used_bytes:0,
                        used_percent:0,
                        max_bytes:0,
                        message: output,
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

            (GET) (/status) => {
                rouille::Response::text(" ").with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/version) => {
                rouille::Response::text(VERSION).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            _ => rouille::Response::empty_404()
        )
    });
}

fn run_cli_command(args:Vec<&str>)->String{
    run_cli_command_with_path(args,"")
}

fn run_cli_command_with_path(args:Vec<&str>,dir:&str)->String{
    let mut str = String::new();
    let mut binding = Command::new("arduino-cli")
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(format!("{SKETCHES_FOLDER}/{dir}"))
        .arg("--no-color")
        .args(args)
        .output()
        .unwrap();
    let stdout = binding
        .stderr
        .as_mut_slice();

    std::str::from_utf8(&stdout).unwrap().to_string()
}