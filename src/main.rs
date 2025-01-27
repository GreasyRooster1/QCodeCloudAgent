use std::string::ToString;
use rouille::router;
use serde::Serialize;

const PORT:i32 = 8181;
const SKETCH_FOLDER:&str = "./sketches";

#[derive(Serialize)]
struct CreateResponse{
    ok:bool
}

fn main() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,

            (GET) (/create/{id:String}) => {
                let mut response = &CreateResponse{
                    ok:true,
                };

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