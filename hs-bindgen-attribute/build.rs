//! Enable proc-macro diagnostics by default when toolchain is set on nightly!

fn main() {
    if let Ok(v) = rustc_version::version_meta() {
        if v.channel == rustc_version::Channel::Nightly {
            println!("cargo:rustc-cfg=DIAGNOSTICS");
        }
    }
}
