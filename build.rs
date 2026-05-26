fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // ── kadr.exe ──────────────────────────────────────────────────────────────
    let exe_dest = format!("{out_dir}/kadr_embedded.exe");
    let exe_src = std::env::var("KADR_EXE_SRC")
        .unwrap_or_else(|_| "../target/release/kadr.exe".to_owned());

    if std::path::Path::new(&exe_src).exists() {
        std::fs::copy(&exe_src, &exe_dest).expect("copy kadr.exe");
        println!("cargo:rerun-if-changed={exe_src}");
    } else {
        std::fs::write(&exe_dest, b"KADR_STUB_NOT_BUILT").expect("write stub");
        println!("cargo:warning=kadr.exe not found at `{exe_src}`. Run: cargo build --release -p kadr first.");
    }

    println!("cargo:rerun-if-env-changed=KADR_EXE_SRC");
    println!("cargo:rustc-env=KADR_EXE_PATH={exe_dest}");

    // ── libmpv-2.dll ──────────────────────────────────────────────────────────
    let dll_dest = format!("{out_dir}/libmpv-2_embedded.dll");
    let dll_src = std::env::var("MPV_DLL_SRC")
        .unwrap_or_else(|_| "../target/release/libmpv-2.dll".to_owned());

    if std::path::Path::new(&dll_src).exists() {
        std::fs::copy(&dll_src, &dll_dest).expect("copy libmpv-2.dll");
        println!("cargo:rerun-if-changed={dll_src}");
    } else {
        std::fs::write(&dll_dest, b"MPV_DLL_STUB").expect("write dll stub");
        println!("cargo:warning=libmpv-2.dll not found at `{dll_src}`. Place the DLL at ../target/release/libmpv-2.dll or set MPV_DLL_SRC.");
    }

    println!("cargo:rerun-if-env-changed=MPV_DLL_SRC");
    println!("cargo:rustc-env=MPV_DLL_PATH={dll_dest}");

    // Windows manifest + version info
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set("ProductName", "Kadr Installer");
        res.set("FileDescription", "Kadr Image Viewer Installer");
        res.set("LegalCopyright", "");
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>
"#);
        let _ = res.compile();
    }
}
