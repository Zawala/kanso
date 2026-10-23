//! Deflates the JavaScript bundles into the build's output folder: MathJax's
//! 1.8 MB is about 0.6 MB in the binary, and Pele's 0.7 MB about 0.2 MB.

use std::io::Write as _;
use std::path::PathBuf;

/// Deflates `sources`, joined in order, into `OUT_DIR/name`.
fn deflate(sources: &[&str], name: &str) {
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    for source in sources {
        println!("cargo:rerun-if-changed={source}");
        let js = std::fs::read(source).unwrap_or_else(|_| panic!("{source} is in assets"));
        encoder.write_all(&js).expect("deflating to memory");
        encoder.write_all(b"\n;\n").expect("deflating to memory");
    }
    let deflated = encoder.finish().expect("deflating to memory");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::write(out.join(name), deflated).expect("writing to OUT_DIR");
}

fn main() {
    deflate(
        &["assets/mathjax/mathjax-tex-svg.js"],
        "mathjax-tex-svg.js.deflate",
    );
    deflate(
        &["assets/pele/pele.global.js", "assets/pele/entry.js"],
        "pele.js.deflate",
    );
}
