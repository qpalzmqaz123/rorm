fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "fcss")]
    build_proto()?;

    Ok(())
}

fn build_proto() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/drivers/fcss/grpc/fcss.proto");

    tonic_build::configure()
        .out_dir("src/drivers/fcss/grpc")
        .compile(
            &["src/drivers/fcss/grpc/fcss.proto"],
            &["src/drivers/fcss/grpc"],
        )?;

    Ok(())
}
