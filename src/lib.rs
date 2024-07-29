use base_request::{RequestResult, TestContext};
use libc::c_char;
use std::ffi::CStr;

pub mod base_cli;
pub mod base_request;

#[no_mangle]
pub extern "C" fn haskell_binding(
    content: *const c_char,
    collection_id: *const c_char,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    let c_str: &CStr = unsafe { CStr::from_ptr(content) };
    let str_slice: &str = c_str.to_str().unwrap();
    let cont_rs: String = str_slice.to_owned();
    let ctx = TestContext {
        file: "haskell_binding".into(),
        file_source: cont_rs.clone(),
        ..Default::default()
    };
    let col = unsafe { CStr::from_ptr(collection_id) };
    let col_str = Some(col.to_str().unwrap().to_owned());
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { base_request::run_json(ctx, cont_rs, false, col_str).await });
    println!("haskell_binding result: {:?}", result);

    return result;
}
