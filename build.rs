extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["./sgcp/sgcp.proto"],
                                &["./sgcp"]).unwrap();
}