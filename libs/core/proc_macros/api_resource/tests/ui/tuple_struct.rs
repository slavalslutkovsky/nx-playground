use core_proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(collection = 123)]
pub struct User {
    id: String,
}

fn main () {}
