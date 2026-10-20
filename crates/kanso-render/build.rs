//! Deflates the MathJax bundle into the build's output folder: 1.8 MB of
//! JavaScript is about 0.6 MB in the binary.

use std::io::Write as _;
use std::path::PathBuf;

fn main() {
    let source = "assets/mathjax/mathjax-tex-svg.js";
    println!("cargo:rerun-if-changed={source}");
    let js = std::fs::read(source).expect("the MathJax bundle is in assets/mathjax");
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&js).expect("deflating to memory");
    let deflated = encoder.finish().expect("deflating to memory");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::write(out.join("mathjax-tex-svg.js.deflate"), deflated).expect("writing to OUT_DIR");
}
