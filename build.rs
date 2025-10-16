use std::env;
use std::path::PathBuf;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    if !target_arch.starts_with("riscv") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("libs");

    if !lib_dir.exists() {
        panic!("libs directory not found! Please ensure the repository is properly cloned.");
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");

    let milkv_libs = [
        "milkv_stream", "sys", "vi", "vo", "vpss", "gdc", "rgn", "ini",
        "sns_full", "sample", "sample_rtsp", "isp", "vdec", "venc", 
        "awb", "ae", "af", "cvi_bin_isp", "cvi_bin", "cvi_rtsp", 
        "misc", "isp_algo", "cvikernel", "cvimath", "cviruntime", 
        "cvi_ive"
    ];

    let opencv_libs = [
        "opencv_core", "opencv_imgcodecs", "opencv_imgproc"
    ];

    let other_libs = [
        "z"  // zlib
    ];

    for lib in milkv_libs.iter().chain(opencv_libs.iter()).chain(other_libs.iter()) {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    // println!("cargo:rustc-link-lib=dylib=stdc++");
    // println!("cargo:rustc-link-lib=dylib=atomic");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libs");
}

