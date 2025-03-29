use std::{fs, thread};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use rouille::router;
use serde_json::Value;
use crate::{CommandOutput, GENERIC_OK};

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const PYTHON_PORT:i32 = 8383;
const PYTHON_FOLDER:&str = "./python";
const PYTHON_VERSION:&str = "1.0.0";

const SERIALIZED_SYSTEM_NAME:&str = "__serialized_filesystem.internal.json";
const LOG_SYSTEM_NAME:&str = "__log_output.internal.log";

pub fn start_python() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PYTHON_PORT}"), move |request| {
        router!(request,
            (POST) (/deserialize/{name:String}) => {
                let path =  format!("{PYTHON_FOLDER}/{name}/");
                let system_path =  format!("{PYTHON_FOLDER}/{name}/{SERIALIZED_SYSTEM_NAME}");
                let content: String = fs::read_to_string(&system_path).unwrap();

                let mut system_json: Value = serde_json::from_str(content.as_str()).unwrap();
                deserialize_filesystem(&mut system_json,path);

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/write/{name:String}) => {
                let path =  format!("{PYTHON_FOLDER}/{name}/");
                let system_path =  format!("{PYTHON_FOLDER}/{name}/{SERIALIZED_SYSTEM_NAME}");
                let mut buffer = String::new();
                fs::create_dir_all(Path::new(&system_path).parent().unwrap()).unwrap();

                fs::remove_dir_all(&path).unwrap();
                fs::create_dir(&path).unwrap();

                let mut file = File::create(&system_path).unwrap();
                request.data().unwrap().read_to_string(&mut buffer).unwrap();
                file.write_all(buffer.as_bytes()).unwrap();

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/execute/{name:String}) => {
                run_command("pip".to_string(),vec!["-r","requirements.txt"],format!("{PYTHON_FOLDER}/{name}/").as_str());

                thread::spawn(move || {

                    let stdout = Command::new("python")
                        .creation_flags(CREATE_NO_WINDOW)
                        .arg("main.py")
                        .current_dir(format!("{PYTHON_FOLDER}/{name}/"))
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap()
                        .stdout
                        .ok_or_else(|| "Could not capture standard output.");

                    let reader = BufReader::new(stdout.unwrap());
                    let log_path = format!("{PYTHON_FOLDER}/{name}/{LOG_SYSTEM_NAME}");
                    File::create(&log_path).expect("Could not create file");
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(&log_path)
                        .unwrap();
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| writeln!(file, "{line}").unwrap());

                });

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/status) => {
                rouille::Response::text(" ").with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/version) => {
                rouille::Response::text(PYTHON_VERSION).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            _ => rouille::Response::empty_404()
        )
    });
}

fn deserialize_filesystem(folder:&mut Value,path:String) {
    fs::create_dir_all(Path::new(&path)).unwrap();
    for (key, val) in folder.as_object_mut().unwrap() {
        let parsed_key = key.replace("➽",".");
        if val.is_string() {
            let mut file = File::create(format!("{}/{}", &path, &parsed_key)).unwrap();
            file.write_all(val.as_str().unwrap().as_bytes()).unwrap();
        }
        if val.is_object() {
            deserialize_filesystem(val, format!("{}/{}", &path, &parsed_key));
        }
    }
}

fn run_command(command: String, args: Vec<&str>, dir:&str) -> CommandOutput {
    let mut str = String::new();
    let mut binding = Command::new(command)
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(dir)
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