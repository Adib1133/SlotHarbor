fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") || !cfg!(windows) {
        panic!("This SlotHarbor build requires a Windows host and Windows target.");
    }

    println!("cargo:rerun-if-env-changed=LUDUSAVI_VERSION");
    println!("cargo:rerun-if-env-changed=LUDUSAVI_VARIANT");

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/windows/manifest.rc");
        println!("cargo:rerun-if-changed=assets/windows/manifest.xml");
        println!("cargo:rerun-if-changed=assets/icon.ico");

        let version = env!("CARGO_PKG_VERSION");
        let version_parts = format!(
            "{},{},{},0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
        );
        embed_resource::compile(
            "assets/windows/manifest.rc",
            [
                format!("SLOTHARBOR_VERSION_PARTS={version_parts}"),
                format!(r#"SLOTHARBOR_VERSION_STRING="{version}""#),
            ],
        )
        .manifest_required()
        .unwrap();
    }
}
