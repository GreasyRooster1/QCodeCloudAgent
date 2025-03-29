use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use rouille::router;
use serde_json::Value;
use crate::{CommandOutput, GENERIC_OK};

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const PYTHON_PORT:i32 = 8383;
const PYTHON_FOLDER:&str = "./python";
const PYTHON_VERSION:&str = "1.0.0";

const SERIALIZED_SYSTEM_NAME:&str = "__serialized_filesystem.internal.json";

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
                let main_file =  format!("{PYTHON_FOLDER}/{name}/main.py");
                run_command("python".to_string(),vec![main_file.as_str()],format!("{PYTHON_FOLDER}/{name}/").as_str());

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

fn run_command(command: String,args: Vec<&str>,dir:&str){
    let mut str = String::new();
    let mut binding = Command::new(command)
        //.creation_flags(CREATE_NO_WINDOW)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
}