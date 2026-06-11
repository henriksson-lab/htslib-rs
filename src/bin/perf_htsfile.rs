fn main() {
    let argv: Vec<Vec<u8>> = std::env::args().map(|arg| arg.into_bytes()).collect();
    let status = unsafe { htslib_rs::htsfile::htsfile_c_227_main(argv) };
    std::process::exit(status);
}
