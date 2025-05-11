use std::{fs, thread};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use rouille::router;
use serde::Serialize;
use serde_json::Value;
use crate::{CommandOutput, GENERIC_OK};

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const PYTHON_PORT:i32 = 8383;
const PYTHON_FOLDER:&str = "./python";
const PYTHON_VERSION:&str = "1.0.0";

const SERIALIZED_SYSTEM_NAME:&str = "__serialized_filesystem.internal.json";
const LOG_SYSTEM_NAME:&str = "__log_output.internal.log";

const TEXT_FILE_TYPES: [&str; 21] = ["html","css","js","py","txt","json","bat","cpp","c","rs","ts","jsx","tsx","sh","dat","yaml","toml","xml","","http",""];

#[derive(Serialize)]
struct LogResponse {
    success:bool,
    logs:String,
}


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
                run_command("pip".to_string(),vec!["install","-r","requirements.txt"],format!("{PYTHON_FOLDER}/{name}/").as_str());

                thread::spawn(move || {
                    let binding = Command::new("python")
                        .creation_flags(CREATE_NO_WINDOW)
                        .arg("main.py")
                        .current_dir(format!("{PYTHON_FOLDER}/{name}/"))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .unwrap();
                    let stdout = binding
                        .stdout
                        .ok_or_else(|| "Could not capture standard output.");
                    let stderr = binding
                        .stderr
                        .ok_or_else(|| "Could not capture standard error.");

                    let out_reader = BufReader::new(stdout.unwrap());
                    let err_reader = BufReader::new(stderr.unwrap());

                    let log_path = format!("{PYTHON_FOLDER}/{name}/{LOG_SYSTEM_NAME}");
                    File::create(&log_path).expect("Could not create file");

                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(&log_path)
                        .unwrap();
                    out_reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            writeln!(file, "{line}").unwrap();
                            println!("{line}")
                    });

                    err_reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            writeln!(file, "{line}").unwrap();
                            println!("{line}")
                    });

                });

                rouille::Response::json(&GENERIC_OK).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (POST) (/log/{name:String}) => {
                let log_path = format!("{PYTHON_FOLDER}/{name}/{LOG_SYSTEM_NAME}");
                println!("{}" ,log_path);
                rouille::Response::json(&LogResponse{
                    success: true,
                    logs: fs::read_to_string(log_path).unwrap().to_string(),
                }).with_additional_header("Access-Control-Allow-Origin", "*")
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
            deserialize_file(&path, &parsed_key,&val.as_str().unwrap());
        }
        if val.is_object() {
            deserialize_filesystem(val, format!("{}/{}", &path, &parsed_key));
        }
    }
}

fn deserialize_file(path:&str,name:&str,content:&str) {
    let mut file = File::create(format!("{}/{}", &path, &name)).unwrap();
    let extension = Path::new(name).extension().unwrap().to_str().unwrap();
    if !TEXT_FILE_TYPES.contains(&extension) {
        //thread::spawn(move ||{
            reqwest::blocking::get(content)
                .unwrap()
                .copy_to(&mut file)
                .unwrap();
        //});
    }else {
        file.write_all(content.as_bytes()).unwrap();
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