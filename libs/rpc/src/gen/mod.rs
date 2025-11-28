// @generated
// This file wires up buf-generated protobuf code
// Note: The prost files already include!() the tonic files automatically

pub mod commons {
    include!("commons.rs");
}

pub mod tasks {
    include!("tasks.rs");
    // tasks.tonic.rs is auto-included by tasks.rs
}

pub mod users {
    include!("users.rs");
    // users.tonic.rs is auto-included by users.rs
}

pub mod terran {
    include!("terran.v1.rs");
    // terran.v1.tonic.rs is auto-included by terran.v1.rs
}
