//! Provides a `NullMigrator` for settings that do not require migration, e.g. settings with a
//! single version.
use crate::migrate::{MigrationResult, ModelStore};
use crate::model::{AsTypeErasedModel, TypeErasedModel};
use crate::Migrator;
use std::any::Any;

mod extensionbuilder;

pub use error::NullMigratorError;
pub use extensionbuilder::NullMigratorExtensionBuilder;

/// `NullMigrator` is to be used for settings that do not require migration, e.g. settings with a
/// single version. For cases where multiple versions of a setting are required, you should use a
/// different Migrator, such as `LinearMigrator`, and define migrations between each version.
///
/// As `NullMigrator` takes anything that implements `TypeErasedModel`, it can be used with any
/// existing `SettingsModel` without needing to implement any additional traits.
#[derive(Default, Debug, Clone)]
pub struct NullMigrator;

impl Migrator for NullMigrator {
    type ErrorKind = NullMigratorError;
    type ModelKind = Box<dyn TypeErasedModel>;

    /// Asserts that the `NullMigrator` is only used with a single version of a model. For cases
    /// where multiple versions are required, you should use a different migrator, such as
    /// `LinearMigrator`.
    fn validate_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
    ) -> Result<(), Self::ErrorKind> {
        snafu::ensure!(models.len() == 1, error::TooManyModelVersionsSnafu);
        Ok(())
    }

    /// Always returns a `NoMigration` error. Extensions that use `NullMigrator` should never need
    /// to migrate.
    fn perform_migration(
        &self,
        _models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        _starting_value: Box<dyn Any>,
        _starting_version: &str,
        _target_version: &str,
    ) -> Result<serde_json::Value, Self::ErrorKind> {
        Err(NullMigratorError::NoMigration)
    }

    /// Always returns a `NoMigration` error. Extensions that use `NullMigrator` should never need
    /// to migrate.
    fn perform_flood_migrations(
        &self,
        _models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        _starting_value: Box<dyn Any>,
        _starting_version: &str,
    ) -> Result<Vec<MigrationResult>, Self::ErrorKind> {
        Err(NullMigratorError::NoMigration)
    }
}

// Needed to satisfy the type constraints of `ModelKind` in `Migrator`. Unfortunately, `Box` has no
// way of providing all traits implemented by the type it points to, so we need to reimplement this
// trait ourselves.
impl AsTypeErasedModel for Box<dyn TypeErasedModel> {
    fn as_model(&self) -> &dyn TypeErasedModel {
        self.as_ref()
    }
}

mod error {
    #![allow(missing_docs)]
    use snafu::Snafu;

    /// The error type returned by `NullMigrator`.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum NullMigratorError {
        #[snafu(display("No migration to perform"))]
        NoMigration,

        #[snafu(display("NullMigrator cannot be used with models with multiple versions"))]
        TooManyModelVersions,
    }
}
