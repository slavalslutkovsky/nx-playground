use core_proc_macros::ApiResource;

#[derive(ApiResource)]
#[api_resource(unknown_attr = "value")]
pub struct User {
    id: String,
}

fn main() {}
