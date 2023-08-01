
use anyhow::*;

fn main() -> Result<()> {
    #[cfg(feature = "ttrpc-codegen")]
    ttrpc_codegen::Codegen::new()
        .out_dir("./src")
        .input("./proto/sealed_secret.proto")
        .include("./proto")
        .rust_protobuf()
        .customize(ttrpc_codegen::Customize {
            async_all: true,
            ..Default::default()
        })
        .rust_protobuf_customize(ttrpc_codegen::ProtobufCustomize::default().gen_mod_rs(false))
        .run()
        .context("ttrpc build")?;

    Ok(())
}
