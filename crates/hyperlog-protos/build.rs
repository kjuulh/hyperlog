fn main() {
    tonic_build::compile_protos("proto/hyperlog.proto").unwrap();
}
