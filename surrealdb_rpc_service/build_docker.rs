use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::compile_protos("proto/authentication_service.proto")?;
    Ok(())
}
