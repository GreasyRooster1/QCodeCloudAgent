use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
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


fn main() {
    run_cli_command(vec!["core","update-index"]);

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,
            (GET) (/create/{name:String}) => {
                run_cli_command(vec![
                    "sketch",
                    "new",
                    name.as_str(),
                ]);

                rouille::Response::json(&GENERIC_OK)
            },

            (GET) (/compile/{name:String}) => {
                run_cli_command(vec![
                    "compile",
                    "-b",
                    "arduino:avr:uno",
                    "-e",
                    name.as_str(),
                ]);

                rouille::Response::json(&GENERIC_OK)
            },

            (POST) (/write/{name:String}) => {
                let path =  format!("{SKETCHES_FOLDER}/{name}/{name}.ino");
                let mut buffer = String::new();
                let mut file = File::create(&path).unwrap();
                request.data().unwrap().read_to_string(&mut buffer).unwrap();
                file.write_all(buffer.as_bytes()).unwrap();

                rouille::Response::json(&GENERIC_OK)
            },

            _ => rouille::Response::empty_404()
        )
    });
}

fn run_cli_command(args:Vec<&str>)->String{
    let mut str = String::new();
    Command::new("arduino-cli")
        .current_dir(SKETCHES_FOLDER)
        .args(args)
        .spawn()
        .expect("command failed")
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut str)
        .expect("cant collect");
    str
}