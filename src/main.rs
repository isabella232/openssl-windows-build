use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

fn build_for_target(target: &str, arg: &str) -> PathBuf {
    // Set up the compilation environment.
    env::set_var("HOST", "x86_64-pc-windows-msvc");
    let vcvars = Command::new(Path::new("src").join("vcvars.bat")).arg(arg).output().unwrap();
    assert!(vcvars.status.success());
    let output = str::from_utf8(&vcvars.stdout).unwrap();
    for line in output.lines() {
        let mut parts = line.splitn(2, '=');
        if let Some(name) = parts.next() {
            if let Some(value) = parts.next() {
                env::set_var(name, value);
            }
        }
	}

    // Build OpenSSL.
    let out_dir = env::current_dir().unwrap().join("openssl-build").join(target);
    //openssl_src::Build::new().target(target).out_dir(out_dir.clone()).build();
    // Return the path to the /install subdirectory that we care about.
    out_dir.join("install")
}

fn main() -> io::Result<()> {
    if env::var("VCVARSALL_PATH").is_err() {
        panic!("Need to provide VCVARSALL_PATH value with path to \
                vcvarsall.bat from Visual Studio installation");
    }

    let targets = &[
        ("aarch64-pc-windows-msvc", "arm64-windows", "x64_arm64"),
        ("x86_64-pc-windows-msvc", "x64-windows", "x64"),
    ];

    let version = openssl_src::version();
    let mut archive = File::create(&format!("{}-vs2017.zip", version))?;
    let mut zip = zip::ZipWriter::new(&mut archive);
    let options = zip::write::FileOptions::default();

    let mut buffer = Vec::new();
    for &(target, subdir, vcvars_arg) in targets.iter() {
        let built = build_for_target(target, vcvars_arg);
        let prefix = built.clone();
        zip.add_directory(subdir, options)?;
        for entry in walkdir::WalkDir::new(built) {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = PathBuf::from(subdir).join(path.strip_prefix(Path::new(&prefix)).unwrap());
            println!("Adding {}", name.display());
            if path.is_file() {
                zip.start_file_from_path(&name, options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else if name.as_os_str().len() != 0 {
                zip.add_directory_from_path(&name, options)?;
            }
        }
    }
    
    zip.finish()?;
    Ok(())
}
