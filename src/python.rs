use std::os::windows::process::CommandExt;
use std::process::Command;
use rouille::router;
use crate::CommandOutput;

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const PYTHON_PORT:i32 = 8383;
const PYTHON_FOLDER:&str = "./python";
const PYTHON_VERSION:&str = "1.0.2";


pub fn start_python() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PYTHON_FOLDER}"), move |request| {
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

fn run_command(command: String,args: Vec<&str>,dir:&str)->CommandOutput {
    let mut str = String::new();
    let mut binding = Command::new(command)
        .creation_flags(CREATE_NO_WINDOW)
        .current_dir(format!("{PYTHON_FOLDER}/{dir}"))
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