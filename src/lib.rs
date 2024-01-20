use base_request::{RequestResult, TestContext};

mod base_cli;
mod base_request;

#[no_mangle]
pub extern "C" fn haskell_binding(
    content: String,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    let ctx = TestContext {
        file: "haskell_binding".into(),
        file_source: content.clone(),
        ..Default::default()
    };
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { base_request::run(ctx, content, false).await });

    return result;
}
