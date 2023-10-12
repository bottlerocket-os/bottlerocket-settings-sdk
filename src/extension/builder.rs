//! Provides a [`SettingsExtensionBuilder`] used to construct [`SettingsExtension`]s.
//! This module also provides the [`extension_builder!`] macro, which can be used to create custom
//! extension builders which are coupled to a specific [`Migrator`].
//!
//! # Examples
//!
//! ```
//! # use bottlerocket_settings_sdk::example::empty::EmptySetting;
//! # use bottlerocket_settings_sdk::extension::SettingsExtensionBuilder;
//! # use bottlerocket_settings_sdk::{
//! #     SettingsExtension, LinearMigrator, BottlerocketSetting,
//! # };
//! # type MySettingV1 = EmptySetting;
//! # type MySettingV2 = EmptySetting;
//! let settings_extension = SettingsExtensionBuilder::new("example", LinearMigrator)
//!     .with_models(vec![
//!         BottlerocketSetting::<MySettingV1>::model(),
//!         BottlerocketSetting::<MySettingV2>::model(),
//!     ])
//!     .build();
//! ```
use super::SettingsExtensionError;
use crate::model::erased::AsTypeErasedModel;
use crate::{Migrator, SettingsExtension};
use tracing::instrument;

#[macro_export]
/// Constructs a [`SettingsExtension`] builder which is associated with a specific migrator.
///
/// To create a builder which uses the [`LinearMigrator`](crate::LinearMigrator), you could use the
/// macro like so:
///
/// ```
/// # use bottlerocket_settings_sdk::extension_builder;
/// # use bottlerocket_settings_sdk::example::empty::EmptySetting;
/// # use bottlerocket_settings_sdk::{SettingsExtension, LinearMigrator, BottlerocketSetting};
/// # type MySettingV1 = EmptySetting;
/// # type MySettingV2 = EmptySetting;
///
/// extension_builder!(
///     pub,
///     MyLinearMigratorExtensionBuilder,
///     LinearMigrator,
///     LinearMigrator
/// );
///
/// let settings_extension = MyLinearMigratorExtensionBuilder::with_name("example")
///     .with_models(vec![
///         BottlerocketSetting::<MySettingV1>::model(),
///         BottlerocketSetting::<MySettingV2>::model(),
///     ])
///     .build();
/// ```
macro_rules! extension_builder {
    ($vis:vis, $builder_name:ident, $migrator:ty, $construct_migrator:expr) => {
        /// Constructs a `SettingsExtension` configured to use the [`$migrator`].
        $vis struct $builder_name(
            $crate::extension::SettingsExtensionBuilder<
                $migrator,
                <$migrator as $crate::Migrator>::ModelKind,
            >,
        );

        impl $builder_name {
            /// Constructs a `SettingsExtension` builder with the given name.
            $vis fn with_name(name: &'static str) -> Self {
                let inner_builder =
                    $crate::extension::SettingsExtensionBuilder::new(name, $construct_migrator);

                Self(inner_builder)
            }

            /// Uses the given set of models for the constructed `SettingsExtension`.
            $vis fn with_models(
                self,
                models: Vec<<$migrator as $crate::Migrator>::ModelKind>,
            ) -> Self {
                let Self(inner_builder) = self;
                let inner_builder = inner_builder.with_models(models);

                Self(inner_builder)
            }

            /// Constructs a `SettingsExtension` with the given options.
            $vis fn build(
                self,
            ) -> Result<
                $crate::extension::SettingsExtension<
                    $migrator,
                    <$migrator as $crate::Migrator>::ModelKind,
                >,
                $crate::extension::SettingsExtensionError<
                    <$migrator as $crate::Migrator>::ErrorKind,
                >,
            > {
                let Self(inner_builder) = self;

                inner_builder.build()
            }
        }
    };
}

/// A builder which can construct a [`SettingsExtension`].
pub struct SettingsExtensionBuilder<Mi, Mo>
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
    /// Returns a new [`SettingsExtensionBuilder`] associated with a given [`Migrator`].
    pub fn new(name: &'static str, migrator: Mi) -> Self {
        Self {
            name,
            migrator,
            models: None,
        }
    }

    /// Set the models for the [`SettingsExtension`].
    pub fn with_models(mut self, models: Vec<Mo>) -> Self {
        self.models = Some(models);
        self
    }

    /// Constructs a [`SettingsExtension`] using the configurations supplied to the builder.
    #[instrument(skip(self), err)]
    pub fn build(self) -> Result<SettingsExtension<Mi, Mo>, SettingsExtensionError<Mi::ErrorKind>> {
        let models = self.models.unwrap_or_default();
        let migrator = self.migrator;

        SettingsExtension::new(self.name, models, migrator)
    }
}
