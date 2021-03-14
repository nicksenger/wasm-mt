//! Utility for testing crates with [`wasm-mt`](https://crates.io/crates/wasm-mt).

#![feature(async_closure)]

use wasm_mt::{WasmMt, utils};
// use wasm_mt::console_ln;
use wasm_bindgen::prelude::*;
use js_sys::ArrayBuffer;

mod transform;
use transform::swc_transform;

// Per crates/web only; TODO generalize for crates/node

pub fn get_pkg_js_uri() -> String {
    let href = utils::run_js("return location.href;").unwrap().as_string().unwrap();
    format!("{}wasm-bindgen-test", href)
}

pub fn pkg_js_no_modules_from(pkg_js: &str) -> String {
    let pkg_js = pkg_js.replace("import.meta.url", "''"); // workaround
    let pkg_js = swc_transform(&pkg_js).unwrap();

    let mut out = String::new();
    out.push_str("const exports = {};");
    out.push_str(&pkg_js);
    out.push_str("const wasm_bindgen = Object.assign(init, exports);");

    out
}

pub async fn create_ab_init(pkg_js_uri: &str) -> Result<ArrayBuffer, JsValue> {
    // let output = swc_transform("let yy = () => {}; export default yy;");
    // console_ln!("output: {:?}", output);
    // assert!(output.unwrap().starts_with("\"use strict\""));

    let pkg_js = utils::fetch_as_text(pkg_js_uri).await?;
    let pkg_js = pkg_js.replace("import.meta.url", "''"); // workaround
    let pkg_js = swc_transform(&pkg_js).unwrap();

    let mut init_js = String::new();
    // init_js.push_str(&fix_nodejs); // TODO in case of tests/crates/node
    init_js.push_str("
        return () => {
            const exports = {};
    ");
    init_js.push_str(&pkg_js);
    init_js.push_str("
            return Object.assign(init, exports);
        };
    ");

    Ok(utils::ab_from_text(&init_js))
}

pub async fn create_mt(pkg_js_uri: &str) -> WasmMt {
    let mt = WasmMt::new(&pkg_js_uri).and_init().await.unwrap();
    let ab = create_ab_init(&pkg_js_uri).await.unwrap();
    mt.set_ab_init(ab);

    mt
}
