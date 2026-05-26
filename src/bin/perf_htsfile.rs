use std::ffi::CString;

fn main() {
    let mut args = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect::<Vec<_>>();
    let mut argv = args
        .iter_mut()
        .map(|arg| arg.as_ptr().cast_mut())
        .collect::<Vec<_>>();
    let status =
        unsafe { htslib_rs::htsfile::htsfile_c_227_main(argv.len() as i32, argv.as_mut_ptr()) };
    std::process::exit(status);
}
