use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(riscv_mode)");
    
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    println!("cargo:DEBUG=milkv-libs build.rs: target_arch={}", target_arch);

    if target_arch.starts_with("riscv") {
        println!("cargo:DEBUG=milkv-libs build.rs: entering riscv mode");
        println!("cargo:rustc-cfg=riscv_mode");
        
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let lib_dir = manifest_dir.join("libs");

        if lib_dir.exists() {
            println!("cargo:info=Setting up library paths for MilkV libraries");
            
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
            
            link_milkv_libraries();
        } else {
            println!("cargo:warning=libs directory not found. Please ensure MilkV libraries are available or enable 'download-libs' feature.");
        }

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=libs");
    } else {
        println!("cargo:DEBUG=milkv-libs build.rs: non-riscv target, skipping library setup");
    }

    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
}

fn link_milkv_libraries() {
    let libraries = [
        "milkv_stream",
        "sys",
        "vi",
        "vo", 
        "vpss",
        "gdc",
        "rgn",
        "ini",
        "sns_full",
        "sample",
        "isp",
        "vdec",
        "venc",
        "awb",
        "ae",
        "af",
        "cvi_bin_isp",
        "cvi_bin",
        "z",
        "cvi_rtsp",
        "misc",
        "isp_algo",
        "cvikernel",
        "cvimath",
        "cviruntime",
        "opencv_core",
        "opencv_imgcodecs", 
        "opencv_imgproc",
        "cvi_ive",
    ];

    for lib in &libraries {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
}
