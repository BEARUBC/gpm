extern crate prost_build;

fn main() {
    let protos = ["./sgcp/sgcp.proto", "./sgcp/bms.proto", "./sgcp/emg.proto", "./sgcp/maestro.proto"];
    prost_build::compile_protos(&protos, &["./sgcp"]).unwrap();
}