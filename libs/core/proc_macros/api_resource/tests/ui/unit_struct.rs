use core_proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(url = true)]
pub struct User {
    id: String,
}

fn main() {}
