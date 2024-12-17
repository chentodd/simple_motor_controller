use micropb_gen::Generator;

fn main() {
    let mut gen = Generator::new();
    gen.add_protoc_arg("--proto_path=../proto")
        .compile_protos(
            &["motor.proto", "sensor.proto", "command.proto"],
            "src/proto_packet.rs",
        )
        .unwrap();
}
