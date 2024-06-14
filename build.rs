extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["src/sgcp/sgcp.proto"],
                                &["src/"]).unwrap();
}