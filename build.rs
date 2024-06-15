extern crate prost_build;

fn main() {
    let protos = ["./sgcp/sgcp.proto", "./sgcp/telemetry.proto", "./sgcp/servo.proto"];
    prost_build::compile_protos(&protos, &["./sgcp"]).unwrap();
}