extern crate rustc_serialize;
#[macro_use] extern crate nickel;

use nickel::{Nickel, HttpRouter, JsonBody};

#[derive(RustcDecodable, RustcEncodable)]
struct Person {
	firstname: String, 
	lastname: String,
}

fn main() {
	let mut server = Nickel::new();

	server.post("/a/post/request", middleware! { |request, response|
		let person = request.json_as::<Person>().unwrap();
		format!("Hello {} {}", person.firstname, person.lastname)
	});

	// server.utilize(router! {
	// 	get "**" => |_req, _res| {
	// 		"Hello world!"
	// 	}
	// });

	server.listen("0.0.0.0:6767");
}
