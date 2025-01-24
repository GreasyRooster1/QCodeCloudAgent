use rouille::router;

const PORT:i32 = 8181;

fn main() {

    // The `start_server` starts listening forever on the given address.
    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,

            (GET) (/create/{id:String}) => {
                rouille::Response::text("hello world")
            },
            (GET) (/{id: String}) => {
                println!("String {:?}", id);
                rouille::Response::text(format!("hello, {}", id))
            },

            _ => rouille::Response::empty_404()
        )
    });
}