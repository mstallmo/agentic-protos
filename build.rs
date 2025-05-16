fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protos/hello_service.proto")?;
    Ok(())
}