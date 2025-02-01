use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use serde::Serialize;

const PORT:i32 = 8181;
const SKETCHES_FOLDER:&str = "./sketches";
const GENERIC_OK:GenericResponse = GenericResponse{
    ok:true
};

#[derive(Serialize)]
struct GenericResponse {
    ok:bool
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

                rouille::Response::json(&GENERIC_OK)
            },

            (POST) (/upload/{name:String}) => {

                let board_out = run_cli_command(vec![
                    "board",
                    "list",
                ]);
                println!("{}", board_out);

                let board_out_words = board_out.split_whitespace().collect::<Vec<&str>>();

                let port = board_out_words[7];

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
                })
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

                if words[0].contains("Used"){
                    return rouille::Response::json(&CompileResponse{
                        success:false,
                        used_bytes:0,
                        used_percent:0,
                        max_bytes:0,
                        message: output,
                    });
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
                })
            },

            (POST) (/write/{name:String}) => {
                let path =  format!("{SKETCHES_FOLDER}/{name}/{name}.ino");
                let mut buffer = String::new();
                let mut file = File::create(&path).unwrap();
                request.data().unwrap().read_to_string(&mut buffer).unwrap();
                file.write_all(buffer.as_bytes()).unwrap();

                rouille::Response::json(&GENERIC_OK)
            },

            (GET) (/status) => {
                rouille::Response::text("")
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
        .current_dir(format!("{SKETCHES_FOLDER}/{dir}"))
        .args(args)
        .output()
        .unwrap();
    let stdout = binding
        .stdout
        .as_mut_slice();

    std::str::from_utf8(&stdout).unwrap().to_string()
}