use std::env;
use std::path::PathBuf;

fn main() {
    // 获取 C API 静态库路径
    let c_api_lib_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..").join("tools").join("c_api_test").join("build")
        .join("c_api").join("Release");

    // 添加库搜索路径
    println!("cargo:rustc-link-search=native={}", c_api_lib_path.display());

    // 链接静态库
    println!("cargo:rustc-link-lib=static=devilutionx_c");

    // 如果 C API 库发生变化,重新构建
    println!("cargo:rerun-if-changed=../tools/c_api/");

    // 生成绑定 (可选 - 需要 bindgen)
    #[cfg(feature = "bindgen")]
    {
        let bindings = bindgen::Builder::default()
            .header("../tools/c_api/devilutionx_c_api.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
