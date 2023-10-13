use crate::migrate::NoMigration;
use crate::SettingsModel;
use std::any::TypeId;

/// A migrator that migrates [`SettingsModel`](crate::SettingsModel)s that implement
/// [`LinearlyMigrateable`] through a linear chain of migrations.
#[derive(Debug, Default, Clone)]
pub struct LinearMigrator;

/// `SettingsModels` that implement `LinearlyMigrateable` can be migrated by following a linear
/// chain of migrations along a defined series of versions.
///
/// For example, consider a settings extensions with versions `[v1, v2, v3, v4, ...]`.
/// When tasked with migrating a setting from `v1` to `v3`, the linear migrator will find that it
/// can do this by first migrating from `v1` to `v2`, then from `v2` to `v3`.
///
/// Invalid migration chains cannot be detected at compile-time; however, the settings sdk checks
/// for validity when an extension is constructed.
///
/// Consider an example in which a setting name has changed:
///
/// ```
/// use bottlerocket_settings_sdk::{LinearlyMigrateable, NoMigration};
/// use serde::{Deserialize, Serialize};
/// use std::collections::HashMap;
///
/// #[derive(Serialize, Deserialize, Debug, Default)]
/// pub struct ScoreV1 {
///     scores: HashMap<String, i64>,
/// }
///
/// #[derive(Serialize, Deserialize, Debug, Default)]
/// pub struct ScoreV2 {
///     all_scores: HashMap<String, i64>,
/// }
///
/// # use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
/// # use bottlerocket_settings_sdk::example::EmptyError;
/// #
/// # type Result<T> = std::result::Result<T, EmptyError>;
/// #
/// # impl SettingsModel for ScoreV1 {
/// #     type PartialKind = Self;
/// #     type ErrorKind = EmptyError;
/// #
/// #     fn get_version() -> &'static str {
/// #         "v1"
/// #     }
/// #
/// #     fn set(_current_value: Option<Self>, target: Self) -> Result<Self> {
/// #         Ok(target)
/// #     }
/// #
/// #     fn generate(
/// #         _: Option<Self::PartialKind>,
/// #         _: Option<serde_json::Value>,
/// #     ) -> Result<GenerateResult<Self::PartialKind, Self>> {
/// #         Ok(GenerateResult::Complete(Self::default()))
/// #     }
/// #
/// #     fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<bool> {
/// #         Ok(true)
/// #     }
/// # }
/// #
/// # impl SettingsModel for ScoreV2 {
/// #     type PartialKind = Self;
/// #     type ErrorKind = EmptyError;
/// #
/// #     fn get_version() -> &'static str {
/// #         "v2"
/// #     }
/// #
/// #     fn set(_current_value: Option<Self>, target: Self) -> Result<Self> {
/// #         Ok(target)
/// #     }
/// #
/// #     fn generate(
/// #         _: Option<Self::PartialKind>,
/// #         _: Option<serde_json::Value>,
/// #     ) -> Result<GenerateResult<Self::PartialKind, Self>> {
/// #         Ok(GenerateResult::Complete(Self::default()))
/// #     }
/// #
/// #     fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<bool> {
/// #         Ok(true)
/// #     }
/// # }
/// impl LinearlyMigrateable for ScoreV1 {
///     type ForwardMigrationTarget = ScoreV2;
///     type BackwardMigrationTarget = NoMigration;
///
///     fn migrate_forward(self) -> Result<ScoreV2> {
///         Ok(ScoreV2 {
///             all_scores: self.scores,
///         })
///     }
///
///     fn migrate_backward(self) -> Result<NoMigration> {
///         NoMigration::no_defined_migration()
///     }
/// }
///
/// impl LinearlyMigrateable for ScoreV2 {
///     type ForwardMigrationTarget = NoMigration;
///     type BackwardMigrationTarget = ScoreV1;
///
///     fn migrate_forward(self) -> Result<NoMigration> {
///         NoMigration::no_defined_migration()
///     }
///
///     fn migrate_backward(self) -> Result<ScoreV1> {
///         Ok(ScoreV1 {
///             scores: self.all_scores,
///         })
///     }
/// }
/// ```
pub trait LinearlyMigrateable: SettingsModel {
    /// The `SettingsModel` that we migrate forward to.
    ///
    /// Can be [`NoMigration`](crate::migrate::NoMigration) to indicate that no forward migraton
    /// exists.
    type ForwardMigrationTarget: LinearlyMigrateable + 'static;

    /// The `SettingsModel` that we migrate backward to.
    ///
    /// Can be [`NoMigration`](crate::migrate::NoMigration) to indicate that no forward migraton
    /// exists.
    type BackwardMigrationTarget: LinearlyMigrateable + 'static;

    /// Returns a string representing the version that this model migrates forward to.
    ///
    /// The default implementation should suffice in almost all circumstances.
    fn migrates_forward_to() -> Option<&'static str> {
        if TypeId::of::<Self::ForwardMigrationTarget>() == TypeId::of::<NoMigration>() {
            None
        } else {
            Some(Self::ForwardMigrationTarget::get_version())
        }
    }

    /// Migrates this settings value forward.
    fn migrate_forward(self) -> Result<Self::ForwardMigrationTarget, Self::ErrorKind>;

    /// Returns a string representing the version that this model migrates backward to.
    ///
    /// The default implementation should suffice in almost all circumstances.
    fn migrates_backward_to() -> Option<&'static str> {
        if TypeId::of::<Self::BackwardMigrationTarget>() == TypeId::of::<NoMigration>() {
            None
        } else {
            Some(Self::BackwardMigrationTarget::get_version())
        }
    }

    /// Migrates this settings value backward.
    fn migrate_backward(self) -> Result<Self::BackwardMigrationTarget, Self::ErrorKind>;
}
