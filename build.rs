use std::{env, fs, path::Path};

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            files.push(name);
        }
    }
}

fn main() {
    // NSIS installs this helper for PATH registration and unregistration. Stage
    // it next to the binary so the template finds it through MAINBINARYSRCPATH.
    println!("cargo:rerun-if-changed=packaging/windows-path.ps1");
    println!("cargo:rerun-if-changed=packaging/icon.ico");
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        // The installer icon does not set the installed executable's icon.
        winresource::WindowsResource::new()
            .set_icon("packaging/icon.ico")
            .compile()
            .expect("failed to embed the Grid9 Windows executable icon");
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let profile_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
        fs::copy(
            "packaging/windows-path.ps1",
            profile_dir.join("grid9.exe.path.ps1"),
        )
        .unwrap();
    }
    let root = Path::new("src/components");
    let mut files = Vec::new();
    for component in ["documentation", "examples"] {
        collect(root, &root.join(component), &mut files);
    }
    let mut generated = String::from("static COMPONENTS: &[(&str, &[u8])] = &[\n");
    for name in files {
        let absolute = fs::canonicalize(root.join(&name)).unwrap();
        generated.push_str(&format!(
            "({name:?}, include_bytes!({:?})),\n",
            absolute.to_str().unwrap()
        ));
    }
    generated.push_str("];\n");
    fs::write(
        Path::new(&env::var_os("OUT_DIR").unwrap()).join("components.rs"),
        generated,
    )
    .unwrap();
}
