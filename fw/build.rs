use micropb_gen::Generator;

fn main() {
    let mut gen = Generator::new();
    gen.use_container_heapless()
        .add_protoc_arg("--proto_path=../proto")
        .compile_protos(&["command.proto"], "src/proto_packet.rs")
        .unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
