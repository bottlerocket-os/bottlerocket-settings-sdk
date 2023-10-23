//! Provides types for creating custom helper functions for use in Bottlerocket's templating engine.
//!
//! See the documentation of [`HelperDef`] for more information.
pub use bottlerocket_template_helper::template_helper;

/// This trait allows users to create custom helper functions for use in Bottlerocket's templating
/// configuration system.
///
/// Helpers are used to run arbitrary rendering code when writing config files.
///
/// # Helper Definitions
/// Any type that implements [`HelperDef`] can be used as a helper. You can use the
/// [`template_helper`] annotation to generate a function that implements [`HelperDef`] for you, so
/// long as:
/// * Your function arguments implement [`serde::Deserialize`]
/// * Your return value is a `Result<T, E>` where `T` implements [`serde::Serialize`]
///   and `E` implements `Into<Box<dyn std::error::Error>>`.
///
/// # Example
///
/// ```
/// use bottlerocket_settings_sdk::helper::{HelperDef, template_helper};
/// use serde_json::json;
///
/// #[template_helper(ident = join_strings_helper)]
/// fn join_strings(lhs: String, rhs: String) -> Result<String, anyhow::Error> {
///     Ok(format!("{}{}", lhs, rhs))
/// }
///
/// assert_eq!(
///     join_strings_helper.helper_fn(vec![json!("hello "), json!("world")]).unwrap(),
///     json!("hello world")
/// );
///
/// ```
pub trait HelperDef {
    /// Executes the helper.
    ///
    /// All inputs are provided as a list of JSON values, and a resulting JSON value is expected as
    /// output.
    fn helper_fn(&self, args: Vec<serde_json::Value>) -> Result<serde_json::Value, HelperError>;
}

impl<F: Fn(Vec<serde_json::Value>) -> Result<serde_json::Value, HelperError>> HelperDef for F {
    fn helper_fn(&self, args: Vec<serde_json::Value>) -> Result<serde_json::Value, HelperError> {
        self(args)
    }
}

#[macro_export]
/// Creates a map of helper names to helper definitions.
///
/// This macro is useful for providing template helpers from a settings model:
///
/// ```
/// # use std::collections::HashMap;
/// use bottlerocket_settings_sdk::{
///     HelperDef, provide_template_helpers, template_helper};
///
/// #[template_helper(ident = exclaim_helper)]
/// fn exclaim(s: String) -> Result<String, anyhow::Error> {
///     Ok(format!("{}!", s))
/// }
///
/// fn template_helpers() -> HashMap<String, Box<dyn HelperDef>> {
///     provide_template_helpers! {
///         "exclaim" => exclaim_helper,
///     }
/// }
/// ```
macro_rules! provide_template_helpers {
    ($($helper_name:expr => $helper:ident),* $(,)?) => {
        {
            let mut helpers = std::collections::HashMap::new();
            $(
                helpers.insert(
                    $helper_name.to_string(),
                    Box::new($helper)as Box<dyn bottlerocket_settings_sdk::HelperDef>
                );
            )*
            helpers
        }
    };
}

mod error {
    #![allow(missing_docs)]
    use snafu::Snafu;

    /// Error type used in helper definitions.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum HelperError {
        #[snafu(display(
            "Helper called with incorrect arity: expected {} args, but {} provided",
            expected_args,
            provided_args
        ))]
        Arity {
            expected_args: usize,
            provided_args: usize,
        },

        #[snafu(display("Failed to execute helper: {}", source))]
        HelperExecute {
            source: Box<dyn std::error::Error + Send + Sync + 'static>,
        },

        #[snafu(display("Failed to parse incoming value from JSON: {}", source))]
        JSONParse { source: serde_json::Error },

        #[snafu(display("Failed to parse outgoing value to JSON: {}", source))]
        JSONSerialize { source: serde_json::Error },
    }
}
pub use error::HelperError;
