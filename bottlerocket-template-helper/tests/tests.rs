use anyhow::Result;
use bottlerocket_settings_sdk::HelperDef;
use bottlerocket_template_helper::template_helper;
use serde_json::json;

#[template_helper(ident = join_strings_helper)]
fn join_strings(lhs: String, rhs: String) -> Result<String> {
    Ok(lhs + &rhs)
}

#[test]
fn call_join_strings() {
    assert_eq!(
        join_strings_helper
            .helper_fn(vec![json!("hello "), json!("world!")])
            .unwrap(),
        json!("hello world!"),
    );

    assert!(join_strings_helper(vec![json!("too"), json!("many"), json!("args")]).is_err());

    assert!(join_strings_helper
        .helper_fn(vec![json!("too"), json!("many"), json!("args")])
        .is_err());

    assert!(join_strings_helper(vec![json!("too few args")]).is_err());
}

#[template_helper(ident = no_args_helper)]
fn no_args() -> Result<String> {
    Ok(String::new())
}

#[test]
fn call_no_args() {
    assert_eq!(no_args_helper(vec![]).unwrap(), json!(""));
    assert!(no_args_helper(vec![json!("sneaky arg")]).is_err());
}
