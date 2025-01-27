use rouille::router;

const PORT:i32 = 8181;

struct CreateResponse{
    ok:bool
}

fn main() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,

            (GET) (/create/{id:String}) => {
                rouille::Response::json()
            },
            (GET) (/{id: String}) => {
                println!("String {:?}", id);
                rouille::Response::text(format!("hello, {}", id))
            },

            _ => rouille::Response::empty_404()
        )
    });
}