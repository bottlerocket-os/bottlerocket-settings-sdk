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
/// Invalid migration chains cannot be detected at compile-time; however, the settings sdk provides
/// a mechanism for checking the validity of this chain in tests.
//
// TODO: add example https://github.com/bottlerocket-os/bottlerocket-settings-sdk/issues/2
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
