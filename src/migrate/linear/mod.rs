//! Provides the [`LinearlyMigrateable`] trait that is needed to use the [`LinearMigrator`] with a
//! [`SettingsModel`](crate::model::SettingsModel).
use super::{Migrator, ModelStore, NoMigration};
use erased::TypeErasedLinearlyMigrateable;
use snafu::OptionExt;
use std::any::Any;
use std::fmt::Debug;
use tracing::{debug, instrument};
use MigrationDirection::{Backward, Forward};

mod erased;
mod interface;
pub use error::LinearMigratorError;
pub use interface::{LinearMigrator, LinearlyMigrateable};

/// The concrete type that the linear migrator manages.
pub type LinearMigratorModel = Box<dyn TypeErasedLinearlyMigrateable>;

impl Migrator for LinearMigrator {
    type ModelKind = LinearMigratorModel;
    type ErrorKind = LinearMigratorError;

    /// Asserts that a linear migration chain exists which includes all models.
    fn validate_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
    ) -> Result<(), LinearMigratorError> {
        let starting_point = models.iter().next();

        if let Some((mut _version, mut _model)) = starting_point {
            // Don't forget to detect loops
            todo!("https://github.com/bottlerocket-os/bottlerocket-settings-sdk/issues/2")
        } else {
            Ok(())
        }
    }

    /// Migrates data from a starting version to a target version.
    ///
    /// The `LinearMigrator` checks that a migration chain exists between the two given versions, then iteratively
    /// migrates the data through that chain until it is the desired version.
    #[instrument(skip(self, models), err)]
    fn perform_migration(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        starting_value: Box<dyn Any>,
        starting_version: &str,
        target_version: &str,
    ) -> Result<serde_json::Value, LinearMigratorError> {
        debug!(starting_version, target_version, "Starting migration.",);

        let starting_model =
            models
                .get_model(starting_version)
                .context(error::NoSuchModelSnafu {
                    version: starting_version.to_string(),
                })?;

        let mut migration_route = self
            .find_migration_route(models, starting_version, target_version)
            .context(error::NoMigrationRouteSnafu {
                starting_version: starting_version.to_string(),
                target_version: target_version.to_string(),
            })?;

        debug!(
            starting_version,
            target_version, "Performing all submigrations to satisfy migration."
        );
        // Consume the route of migration directions, keeping track of the data and version as we go
        let result = migration_route
            .try_fold(
                (starting_value, starting_model),
                |(curr_value, curr_model), next_direction| {
                    let current_version = curr_model.as_model().get_version();
                    let next_version = curr_model.migrates_to(next_direction).expect(
                        "Failed to find migration which was previously found during route \
                        selection.",
                    );
                    debug!(current_version, target_version, "Performing submigration.");

                    let next_model = models.get_model(next_version).expect(
                        "Failed to find migration which was previously found during route \
                        selection.",
                    );
                    let next_value = curr_model.migrate(curr_value, next_direction)?;

                    Ok((next_value, next_model))
                },
            )
            .and_then(|(final_value, final_model)| final_model.serialize(final_value));

        debug!(starting_version, target_version, "Migration complete.");

        result
    }
}

/// Iterates through models, following a linear migration chain starting from a given model and moving in a given
/// direction (forwards/backwards).
struct MigrationIter<'a> {
    direction: MigrationDirection,
    models: &'a dyn ModelStore<ModelKind = LinearMigratorModel>,
    current: Option<&'a dyn TypeErasedLinearlyMigrateable>,
}

impl<'a> Iterator for MigrationIter<'a> {
    type Item = &'a dyn TypeErasedLinearlyMigrateable;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        self.current = current
            .migrates_to(self.direction)
            .and_then(|next_version| self.models.get_model(next_version).map(|i| i.as_ref()));

        Some(current)
    }
}

/// Represents the direction for a linear migration.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MigrationDirection {
    /// A migration forward, to a newer version.
    Forward,
    /// A migraton backward, to an older version.
    Backward,
}

impl std::fmt::Display for MigrationDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Forward => "forward",
            Backward => "backward",
        })
    }
}

impl LinearMigrator {
    /// Returns an iterator of migrations to be performed to transform data from a starting version to a target version.
    fn find_migration_route(
        &self,
        all_models: &dyn ModelStore<ModelKind = LinearMigratorModel>,
        starting_version: &str,
        target_version: &str,
    ) -> Option<impl Iterator<Item = MigrationDirection>> {
        debug!(starting_version, target_version, "Finding migration route");

        // This closure searches through the migrations in a given direction. If we find the target version,
        // we return the number of migrations required in the given direction to reach that version.
        let search_in_direction = |direction: MigrationDirection| {
            debug!(starting_version, %direction, "Searching for migration route");

            self.migration_iter(all_models, starting_version, direction)
                .enumerate()
                .find(|(_ndx, model)| model.as_model().get_version() == target_version)
                .map(|(ndx, _)| {
                    debug!(
                        starting_version,
                        target_version, "Migration found: travel {} hops {}.", ndx, direction
                    );
                    (ndx, direction)
                })
                .or_else(|| {
                    debug!(
                        starting_version,
                        target_version,
                        %direction,
                        "No migration route found."
                    );
                    None
                })
        };

        (starting_version == target_version)
            .then_some((0, Forward)) // 0 hops required for "identity" migration
            .or_else(|| search_in_direction(Forward))
            .or_else(|| search_in_direction(Backward))
            .map(|(num_hops, direction)| std::iter::repeat(direction).take(num_hops))
    }

    /// Iterate through the extensions chain of model migrations, starting at a given version.
    fn migration_iter<'a>(
        &self,
        models: &'a dyn ModelStore<ModelKind = LinearMigratorModel>,
        starting_version: &str,
        direction: MigrationDirection,
    ) -> MigrationIter<'a> {
        MigrationIter {
            direction,
            models,
            current: models.get_model(starting_version).map(|i| i.as_ref()),
        }
    }
}

impl LinearlyMigrateable for NoMigration {
    type ForwardMigrationTarget = NoMigration;
    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(self) -> Result<Self::ForwardMigrationTarget, Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }

    fn migrate_backward(self) -> Result<Self::ForwardMigrationTarget, Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }
}

mod error {
    #![allow(missing_docs)]
    use snafu::Snafu;

    use super::MigrationDirection;

    /// Error type returned by the linear migrator.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum LinearMigratorError {
        #[snafu(display("Failed to downcast migrated value as setting version '{}'", version))]
        DowncastSetting { version: &'static str },

        #[snafu(display("No '{}' migration for setting version '{}'", direction, version))]
        NoDefinedMigration {
            direction: MigrationDirection,
            version: &'static str,
        },

        #[snafu(display(
            "No migration route found for '{}' to '{}'",
            starting_version,
            target_version
        ))]
        NoMigrationRoute {
            starting_version: String,
            target_version: String,
        },

        #[snafu(display("Could not find model for version '{}'", version))]
        NoSuchModel { version: String },

        #[snafu(display("Failed to serialize migration result: {}", source))]
        SerializeMigrationResult { source: serde_json::Error },

        #[snafu(display(
            "Failed to perform sub-migration of setting {} from '{}' to '{}': {}",
            direction,
            from_version,
            to_version,
            source
        ))]
        SubMigration {
            from_version: &'static str,
            to_version: &'static str,
            direction: MigrationDirection,
            source: Box<dyn std::error::Error + Send + Sync + 'static>,
        },
    }
}

#[cfg(test)]
mod test {
    use crate::model::erased::TypeErasedModel;
    use maplit::hashmap;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    use super::*;

    // We have to implement a fair few traits to test the migrator.
    /// `FakeModelStore` allows querying for Models of our type `FakeMigrateable`.
    struct FakeModelStore(HashMap<String, Box<dyn TypeErasedLinearlyMigrateable>>);

    impl ModelStore for FakeModelStore {
        type ModelKind = Box<dyn TypeErasedLinearlyMigrateable>;

        fn get_model(&self, version: &str) -> Option<&Self::ModelKind> {
            let Self(inner) = &self;
            inner.get(version)
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (&str, &Self::ModelKind)> + '_> {
            todo!()
        }
    }

    impl<S: AsRef<str>> From<HashMap<S, FakeMigrateable>> for FakeModelStore {
        fn from(value: HashMap<S, FakeMigrateable>) -> Self {
            Self(
                value
                    .into_iter()
                    .map(|(version, model)| {
                        (
                            version.as_ref().to_string(),
                            Box::new(model) as Box<dyn TypeErasedLinearlyMigrateable>,
                        )
                    })
                    .collect(),
            )
        }
    }

    /// `FakeMigrateable` allows constructing arbitrary objects that we can migrate between.
    #[derive(Debug, Serialize, Deserialize)]
    struct FakeMigrateable {
        version: &'static str,
        backward: Option<&'static str>,
        forward: Option<&'static str>,
    }

    impl FakeMigrateable {
        fn new(
            version: &'static str,
            backward: Option<&'static str>,
            forward: Option<&'static str>,
        ) -> Self {
            Self {
                version,
                backward,
                forward,
            }
        }
    }

    // We have to implement `Model` to make `ModelStore` happy
    impl TypeErasedModel for FakeMigrateable {
        fn get_version(&self) -> &'static str {
            self.version
        }

        fn set(
            &self,
            _current: Option<serde_json::Value>,
            _target: serde_json::Value,
        ) -> Result<serde_json::Value, crate::model::BottlerocketSettingError> {
            unimplemented!()
        }

        fn generate(
            &self,
            _existing_partial: Option<serde_json::Value>,
            _dependent_settings: Option<serde_json::Value>,
        ) -> Result<
            crate::GenerateResult<serde_json::Value, serde_json::Value>,
            crate::model::BottlerocketSettingError,
        > {
            unimplemented!()
        }

        fn validate(
            &self,
            _value: serde_json::Value,
            _validated_settings: Option<serde_json::Value>,
        ) -> Result<bool, crate::model::BottlerocketSettingError> {
            unimplemented!()
        }

        fn parse_erased(
            &self,
            _value: serde_json::Value,
        ) -> Result<Box<dyn Any>, crate::model::BottlerocketSettingError> {
            unimplemented!()
        }
    }

    // We ave to implement `TypeErasedLinearlyMigrateable` to make `LinearMigrator` happy.
    impl TypeErasedLinearlyMigrateable for FakeMigrateable {
        fn as_model(&self) -> &dyn crate::model::erased::TypeErasedModel {
            self
        }

        fn migrates_to(&self, direction: MigrationDirection) -> Option<&'static str> {
            match direction {
                Forward => self.forward,
                Backward => self.backward,
            }
        }

        fn migrate(
            &self,
            _current: Box<dyn Any>,
            _direction: MigrationDirection,
        ) -> Result<Box<dyn Any>, LinearMigratorError> {
            todo!()
        }

        fn serialize(
            &self,
            _current: Box<dyn Any>,
        ) -> Result<serde_json::Value, LinearMigratorError> {
            todo!()
        }
    }

    #[test]
    fn test_find_migration_route() {
        let models = FakeModelStore::from(hashmap! {
                "v1" => FakeMigrateable::new("v1", None, Some("v2")),
                "v2" => FakeMigrateable::new("v2", Some("v1"), Some("v3")),
                "v3" => FakeMigrateable::new("v3", Some("v2"), Some("v4")),
                "v4" => FakeMigrateable::new("v4", Some("v3"), Some("v5")),
                "v5" => FakeMigrateable::new("v5", Some("v4"), None),
        });

        [
            ("v3", "v3", Some(vec![])),
            ("v1", "v3", Some(vec![Forward, Forward])),
            ("v1", "v5", Some(vec![Forward, Forward, Forward, Forward])),
            ("v3", "v2", Some(vec![Backward])),
            ("v5", "v2", Some(vec![Backward, Backward, Backward])),
            ("v1", "v7", None),
            ("v9", "definitely-no-such-version", None),
        ]
        .into_iter()
        .for_each(|(start, to, expected)| {
            eprintln!("Testing migration from {} to {}", start, to);
            let migration: Option<Vec<MigrationDirection>> = LinearMigrator
                .find_migration_route(&models, start, to)
                .map(|route| route.collect());

            assert_eq!(migration, expected);
        });
    }

    #[test]
    fn test_migration_iter() {
        let models = FakeModelStore::from(hashmap! {
                "v1" => FakeMigrateable::new("v1", None, Some("v2")),
                "v2" => FakeMigrateable::new("v2", Some("v1"), Some("v3")),
                "v3" => FakeMigrateable::new("v3", Some("v2"), Some("v4")),
                "v4" => FakeMigrateable::new("v4", Some("v3"), Some("v5")),
                "v5" => FakeMigrateable::new("v5", Some("v4"), None),
        });

        let versions = LinearMigrator
            .migration_iter(&models, "v1", Forward)
            .map(|model| model.as_model().get_version())
            .collect::<Vec<_>>();

        assert_eq!(versions, vec!["v1", "v2", "v3", "v4", "v5"])
    }
}
