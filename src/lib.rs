//! Joint compilation of various languages and assembly language.
//! 
//! Just write a target list in build.rs to joint compile your multiple lang project.
//! 
//! A simple example:
//! ```rust
//! use jointcomp::targets;
//! 
//! targets! {
//!     GccAsm {
//!         "main.S",
//!         "test.S",
//!     };
//!     #[cfg(target_arch = "x86_64")]
//!     GccAsm {
//!         "arch/$/foo.S" : "include/$/bar.h",
//!     };
//!     LinkerScript { "myscript-$.lds" };
//!     LinkerMap { "target.map" };
//! }
//! ```

use proc_macro::TokenStream;
use replacing::replace;
use target::from_input;

mod replacing;
mod target;

#[proc_macro]
pub fn targets(input: TokenStream) -> TokenStream {
    let script = from_input(input);
    let target = target().parse::<TokenStream>().unwrap();
    let target = replace(target, &script);
    target
}

fn target() -> String {
    stringify!(
        use std::{collections::HashMap, env::consts::ARCH, path::Path, process::Command};

        #[derive(Debug)]
        enum TargetType {
            GccAsm,
            GccAsmX86,
            LinkerScript,
            LinkerMap,
        }

        impl TargetType {
            fn gen(&self, name: String, source: String, dependencies: Vec<String>) {
                let outdir = std::env::var("OUT_DIR").unwrap();
                match *self {
                    TargetType::GccAsm => {
                        let output1 = Command::new("gcc")
                            .args(&["-c", &source, "-o", &format!("{outdir}/{name}.o")])
                            .status().unwrap();
                        let output2 = Command::new("ar")
                            .args(&["crus", &format!("lib{name}.a"), &format!("{outdir}/{name}.o")])
                            .current_dir(&Path::new(&outdir))
                            .status().unwrap();
                        if !output1.success() || !output2.success() {
                            panic!("Build failed on GccAsm target {name}");
                        }
                        println!("cargo::rustc-link-lib=static={name}");
                        println!("cargo::rerun-if-changed={source}");
                        for s in dependencies {
                            println!("cargo::rerun-if-changed={s}");
                        }
                    }
                    TargetType::GccAsmX86 => {
                        let output1 = Command::new("gcc")
                            .args(&["-c", "-m32", &source, "-o", &format!("{outdir}/{name}_.o")])
                            .status().unwrap();
                        let output1_ = Command::new("objcopy")
                            .args(&[
                                "-O",
                                "elf64-x86-64",
                                &format!("{outdir}/{name}_.o"),
                                &format!("{outdir}/{name}.o")])
                            .status().unwrap();
                        let output2 = Command::new("ar")
                            .args(&["crus", &format!("lib{name}.a"), &format!("{outdir}/{name}.o")])
                            .current_dir(&Path::new(&outdir))
                            .status().unwrap();
                        if !output1.success() || !output1_.success() || !output2.success() {
                            panic!("Build failed on GccAsmX86 target {name}");
                        }
                        println!("cargo::rustc-link-lib=static={name}");
                        println!("cargo::rerun-if-changed={source}");
                        for s in dependencies {
                            println!("cargo::rerun-if-changed={s}");
                        }
                    }
                    TargetType::LinkerScript => {
                        println!("cargo::rustc-link-arg=-T{source}");
                    }
                    TargetType::LinkerMap => {
                        println!("cargo::rustc-link-arg=-Map={outdir}/entry.map");
                    }
                }
            }
        }

        fn main() {
            let outdir = std::env::var("OUT_DIR").unwrap();
            let tarprofs = targets();
            println!("cargo::rustc-link-search=native={outdir}");
            for (name, (ttype, source, deps)) in tarprofs {
                let source = source.replace("$", ARCH);
                let deps = deps.into_iter().map(|s| s.replace("$", ARCH)).collect::<Vec<String>>();
                ttype.gen(name, source, deps);
            }
        }

        fn targets() -> HashMap<String, (TargetType, String, Vec<String>)> {
            let pkgpath = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let mut res = HashMap::new();
            $(
            $code_macro
            {
                let source = format!("{}/src/{}", pkgpath, $source);
                let deps = vec![$($dep.to_string(),)];
                let name = source.split("/").collect::<Vec<_>>();
                let name = name[name.len() - 1];
                res.insert(name.to_string(), ($tartype, source, deps));
                eprintln!("{:?}", $tartype);
            })
            res
        }
    )
    .to_string()
}
