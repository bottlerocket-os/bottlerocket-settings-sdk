//! The user interface to the migrator allows expressing forwards and backwards migration targets
//! using Rust types, e.g.
//! ```ignore
//! impl LinearlyMigrateable for MySettingV1 {
//!     type ForwardMigrationTarget = MySettingV2;
//!     type BackwardMigrationTarget = MySettingV0;
//!     // ...
//! }
//! ```
//! During migration, the migrator must perform a chain of migrations while traversing a linear
//! migration path outlined by these associated types to reach the target.
//! The only way to do this while statically checking type information would involve a combinatoric
//! explosion of migrations, which could be written by a macro but would balloon the binary size.
//! To avoid this, [`SettingsModel`](crate::model::SettingsModel)s are wrapped in a
//! [`BottlerocketSetting`](crate::model::BottlerocketSetting) which provides a quasi-private
//! type-erased interface ([`Model`](crate::model::erased::Model)) over the one defined by settings
//! extension authors.
//!
//! The migrator expands on this private interface via
//! [`TypeErasedLinearlyMigrateable`](self::TypeErasedLinearlyMigrateable).
//!
//! We use the [`Any`] trait to perform type-erasure and downcasting to the associated model types.
use super::interface::LinearlyMigrateable;
use super::{error, LinearMigratorError, MigrationDirection};
use crate::model::erased::{AsTypeErasedModel, TypeErasedModel};
use crate::BottlerocketSetting;
use snafu::{OptionExt, ResultExt};
use std::any::Any;

pub trait TypeErasedLinearlyMigrateable {
    /// Returns the associated model.
    ///
    /// This is a bit of a hack to make it so that `TypeErasedLinearlyMigrateable` trait objects can
    /// blanket implement [`AsModel`].
    fn as_model(&self) -> &dyn TypeErasedModel;

    /// Returns the model version that this model migrates to in a given direction.
    fn migrates_to(&self, direction: MigrationDirection) -> Option<&'static str>;

    /// Accepts a type-erased `BottlerocketSettings` implementor and migrates it in the given
    /// direction.
    fn migrate(
        &self,
        current: &dyn Any,
        direction: MigrationDirection,
    ) -> Result<Box<dyn Any>, LinearMigratorError>;

    /// Serializes a type-erased `BottlerocketSettings`.
    fn serialize(&self, current: &dyn Any) -> Result<serde_json::Value, LinearMigratorError>;
}

impl<T: LinearlyMigrateable + 'static> TypeErasedLinearlyMigrateable for BottlerocketSetting<T> {
    fn as_model(&self) -> &dyn TypeErasedModel {
        self
    }

    fn migrates_to(&self, direction: MigrationDirection) -> Option<&'static str> {
        match direction {
            MigrationDirection::Backward => T::migrates_backward_to(),
            MigrationDirection::Forward => T::migrates_forward_to(),
        }
    }

    fn migrate(
        &self,
        current: &dyn Any,
        direction: MigrationDirection,
    ) -> Result<Box<dyn Any>, LinearMigratorError> {
        let current: &T =
            current
                .downcast_ref()
                .ok_or_else(|| error::LinearMigratorError::DowncastSetting {
                    version: T::get_version(),
                })?;

        match direction {
            MigrationDirection::Backward => {
                let to_version =
                    T::migrates_backward_to().context(error::NoDefinedMigrationSnafu {
                        direction,
                        version: T::get_version(),
                    })?;
                current
                    .migrate_backward()
                    .map_err(Into::into)
                    .context(error::SubMigrationSnafu {
                        from_version: T::get_version(),
                        to_version,
                        direction,
                    })
                    .map(|retval| Box::new(retval) as Box<dyn Any>)
            }
            MigrationDirection::Forward => {
                let to_version =
                    T::migrates_forward_to().context(error::NoDefinedMigrationSnafu {
                        direction,
                        version: T::get_version(),
                    })?;
                current
                    .migrate_forward()
                    .map_err(Into::into)
                    .context(error::SubMigrationSnafu {
                        from_version: T::get_version(),
                        to_version,
                        direction,
                    })
                    .map(|retval| Box::new(retval) as Box<dyn Any>)
            }
        }
    }

    fn serialize(&self, current: &dyn Any) -> Result<serde_json::Value, LinearMigratorError> {
        let current: &T =
            current
                .downcast_ref()
                .ok_or_else(|| error::LinearMigratorError::DowncastSetting {
                    version: T::get_version(),
                })?;
        serde_json::to_value(current).context(error::SerializeMigrationResultSnafu)
    }
}

// We need to implement `AsModel` to satisfy the `SettingsExtension` and `Migrator` interfaces.
// Even if `TypeErasedLinearlyMigrateable` had `AsModel` as a supertrait, supertraits do not extend
// to trait objects.
impl AsTypeErasedModel for Box<dyn TypeErasedLinearlyMigrateable> {
    fn as_model(&self) -> &dyn TypeErasedModel {
        TypeErasedLinearlyMigrateable::as_model(self.as_ref())
    }
}
