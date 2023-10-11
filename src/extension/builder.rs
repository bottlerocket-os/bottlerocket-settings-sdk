//! Provides a [`SettingsExtensionBuilder`](self::SettingsExtensionBuilder) used to construct
//! [`SettingsExtension`](crate::extension::SettingsExtension)s.
//!
//! One unusual quality of the builder is that the [`Migrator`](crate::Migrator) for an extension
//! must be specified first.
//! This behavior is enforced by only providing
//! [`with_migrator`](self::SettingsExtensionBuilder::with_migrator) on
//! [`SettingsExtensionBuilder`], allowing the rest of the builder to be interacted with once it has
//! been called.
//! This design allows the Rust compiler to infer type information about the models without needing
//! users to become aware of some of the gnarlier types in the SDK.
//!
//! # Examples
//!
//! ```
//! # use bottlerocket_settings_sdk::doc_do_not_use::empty::EmptySetting;
//! # use bottlerocket_settings_sdk::{SettingsExtension, LinearMigrator, BottlerocketSetting};
//! # type MySettingV1 = EmptySetting;
//! # type MySettingV2 = EmptySetting;
//! let settings_extension = SettingsExtension::with_name("example")
//!     .with_migrator(LinearMigrator)
//!     .with_models(vec![
//!         BottlerocketSetting::<MySettingV1>::model(),
//!         BottlerocketSetting::<MySettingV2>::model(),
//!     ])
//!     .build();
//! ```
use crate::model::erased::AsTypeErasedModel;
use crate::{Migrator, SettingsExtension};
use snafu::ensure;
use std::collections::HashSet;
use std::marker::PhantomData;
use tracing::{debug, instrument};

/// Builder for `SettingsExtension`.
///
/// You must call `with_migrator()` before further configuring the extension.
#[derive(Debug, Default)]
pub struct SettingsExtensionBuilder<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    name: &'static str,
    _ghost: PhantomData<(Mi, Mo)>,
}

/// Builder for `SettingsExtension`.
pub struct SettingsExtensionBuilderWithMigrator<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    name: &'static str,
    models: Option<Vec<Mo>>,
    migrator: Mi,
}

impl<Mi, Mo> SettingsExtensionBuilder<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    /// Returns a new `SettingsExtensionBuilder`.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _ghost: PhantomData,
        }
    }

    /// Set the migrator for the `SettingsExtension`.
    pub fn with_migrator(self, migrator: Mi) -> SettingsExtensionBuilderWithMigrator<Mi, Mo> {
        SettingsExtensionBuilderWithMigrator::new(self.name, migrator)
    }
}

impl<Mi, Mo> SettingsExtensionBuilderWithMigrator<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    fn new(name: &'static str, migrator: Mi) -> Self {
        Self {
            name,
            migrator,
            models: None,
        }
    }

    /// Set the models for the `SettingsExtension`.
    pub fn with_models(mut self, models: Vec<Mo>) -> Self {
        self.models = Some(models);
        self
    }

    #[instrument(skip(self), err)]
    pub fn build(self) -> Result<SettingsExtension<Mi, Mo>, SettingsExtensionBuilderError> {
        let mut unique_models: HashSet<&str> = HashSet::new();

        // Convert the model vector into a map of Version => Model while checking for version
        // uniqueness.
        debug!("Checking each model for unique versioning.");
        let models = self
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|model| {
                let version = model.as_model().get_version();

                ensure!(
                    !unique_models.contains(version),
                    error::ModelVersionCollisionSnafu {
                        version: version.to_string(),
                    }
                );
                unique_models.insert(version);

                Ok((version.to_string(), model))
            })
            .collect::<Result<_, _>>()?;

        let migrator = self.migrator;

        Ok(SettingsExtension {
            name: self.name,
            models,
            migrator,
        })
    }
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum SettingsExtensionBuilderError {
        #[snafu(display("Models have colliding version '{}'", version))]
        ModelVersionCollision { version: String },
    }
}

pub use error::SettingsExtensionBuilderError;
