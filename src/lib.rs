use base_request::{RequestResult, TestContext};
use libc::c_char;
use std::ffi::{CStr, CString};

mod base_cli;
mod base_request;

#[no_mangle]
pub extern "C" fn haskell_binding(
    content: *const c_char,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    let c_str: &CStr = unsafe { CStr::from_ptr(content) };
    let str_slice: &str = c_str.to_str().unwrap();
    let cont_rs: String = str_slice.to_owned();
    print!("{}", cont_rs);
    let ctx = TestContext {
        file: "haskell_binding".into(),
        file_source: cont_rs.clone(),
        ..Default::default()
    };
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { base_request::run(ctx, cont_rs, false).await });

    return result;
}
