use std::process::Command;
use std::string::ToString;
use rouille::router;
use serde::Serialize;

const PORT:i32 = 8181;
const SKETCHES_FOLDER:&str = "./sketches";

#[derive(Serialize)]
struct CreateResponse{
    ok:bool
}

fn main() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,

            (GET) (/create/{name:String}) => {
                let mut response = &CreateResponse{
                    ok:true,
                };

                run_cli_command(vec![
                    "sketch".to_string(),
                    "new".to_string(),
                    name
                ]);

                rouille::Response::json(response)
            },
            (GET) (/{id: String}) => {
                println!("String {:?}", id);
                rouille::Response::text(format!("hello, {}", id))
            },

            _ => rouille::Response::empty_404()
        )
    });
}

fn run_cli_command(args:Vec<String>){
    Command::new("arduino-cli")
        .current_dir(SKETCHES_FOLDER)
        .args(args)
        .spawn()
        .expect("command failed");
}