# JointComp

Joint compileation of various languages and assembly language.

Just write a target list in build.rs to joint compile your multiple lang project.

A simple example:

```rust
targets! {
    GccAsm {
        "main.S",
        "test.S",
    };
    #[cfg(target_arch = "x86_64")]
    GccAsm {
        "arch/$/foo.S" : "include/$/bar.h",
    };
    LinkerScript { "myscript-$.lds" };
    LinkerMap { "target.map" };
}
```
