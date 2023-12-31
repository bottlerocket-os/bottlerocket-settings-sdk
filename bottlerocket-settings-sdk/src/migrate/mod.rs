//! Provides migrators for moving settings values between versions, such as [`LinearMigrator`].
//! The documentation for these specific migrators is the most useful documentation for most users
//! of this library.
//!
//! The [`Migrator`](self::Migrator) trait, is also provided, which allows settings extensions
//! to customize how they are migrated between different versions.
use crate::model::erased::AsTypeErasedModel;
use crate::{GenerateResult, SettingsModel};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::convert::Infallible;
use std::fmt::Debug;

pub mod linear;
pub use linear::{
    LinearMigrator, LinearMigratorExtensionBuilder, LinearMigratorModel, LinearlyMigrateable,
};

pub mod null;
pub use null::{NullMigrator, NullMigratorExtensionBuilder};

/// Implementors of the `Migrator` trait inform a [`SettingsExtension`](crate::SettingsExtension)
/// how to migrate settings values between different versions.
pub trait Migrator: Debug {
    /// The error type returned by the migrator.
    type ErrorKind: std::error::Error + Send + Sync + 'static;

    /// The type representing stored models.
    ///
    /// This is usually a trait object provided by a [`Migrator`] implementaton; however, the
    /// underlying implementation is almost always a boxed
    /// [`BottlerocketSetting`](crate::BottlerocketSetting).
    type ModelKind: AsTypeErasedModel;

    /// Validates that the given settings extension's models have a coherent linear migration chain.
    ///
    /// Returns an error if there are loops in the migration chain, or if more than one chains exist
    /// in the set of models.
    fn validate_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
    ) -> Result<(), Self::ErrorKind>;

    /// Migrates a given settings value from its starting version to a target version.
    ///
    /// Returns an error if no migration route can be found between the two versions, or if an error
    /// is returned by any migrations defined by the underlying
    /// [`SettingsModel`](crate::SettingsModel).
    fn perform_migration(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        starting_value: Box<dyn Any>,
        starting_version: &str,
        target_version: &str,
    ) -> Result<serde_json::Value, Self::ErrorKind>;

    /// Migrates a given settings value to all other available versions.
    ///
    /// The results from the flood migration include the starting value and version.
    /// Returns an error if one occurs during any migration.
    fn perform_flood_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        starting_value: Box<dyn Any>,
        starting_version: &str,
    ) -> Result<Vec<MigrationResult>, Self::ErrorKind>;
}

/// An individual migration result from a batch migration.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct MigrationResult {
    /// The version resulting from the migration.
    pub version: &'static str,
    /// The value resulting from the migration.
    pub value: serde_json::Value,
}

/// A type that holds settings models, used to resolve version -> model lookups during migrations.
pub trait ModelStore {
    /// The type representing stored models.
    ///
    /// This is usually a trait object provided by a [`Migrator`] implementaton.
    type ModelKind: AsTypeErasedModel;

    /// Retrieves the model for a given version.
    fn get_model(&self, version: &str) -> Option<&Self::ModelKind>;

    /// Iterates over all stored models.
    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &Self::ModelKind)> + '_>;

    /// Returns the number of items stored in the [`ModelStore`].
    fn len(&self) -> usize;

    /// Returns whether or not their are any models in the [`ModelStore`].
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A marker type used to indicate that no migration should be performed.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct NoMigration;

impl NoMigration {
    /// Creates a new `NoMigration` instance.
    pub fn new() -> Self {
        NoMigration
    }

    /// Marker function to call when no migration should be performed.
    ///
    /// These functions should be marked to return `NoMigration`.
    pub fn no_defined_migration<E>() -> Result<Self, E> {
        Ok(NoMigration)
    }
}

// `NoMigration` must implement `SettingsModel` so that it's type can be used as a marker.
// In cases that are parameterized on `SettingsModel` types where `NoMigration` is valid, the
// implementor must check for the presence of `NoMigration` with `TypeId::of`
impl SettingsModel for NoMigration {
    type PartialKind = NoMigration;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<(), Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }

    fn generate(
        _existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>, Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }

    fn validate(
        _value: Self,
        _validated_settings: Option<serde_json::Value>,
    ) -> Result<(), Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }
}
