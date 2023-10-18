//! Provides the [`LinearlyMigrateable`] trait that is needed to use the [`LinearMigrator`] with a
//! [`SettingsModel`](crate::model::SettingsModel).
use super::{MigrationResult, Migrator, ModelStore, NoMigration};
use erased::TypeErasedLinearlyMigrateable;
use snafu::OptionExt;
use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;
use tracing::{debug, instrument};
use MigrationDirection::{Backward, Forward};

mod erased;
mod extensionbuilder;
mod interface;
mod validator;
pub use error::LinearMigratorError;
pub use extensionbuilder::LinearMigratorExtensionBuilder;
pub use interface::{LinearMigrator, LinearlyMigrateable};

/// The concrete type that the linear migrator manages.
pub type LinearMigratorModel = Box<dyn TypeErasedLinearlyMigrateable>;

impl Migrator for LinearMigrator {
    type ModelKind = LinearMigratorModel;
    type ErrorKind = LinearMigratorError;

    /// Asserts that a single linear migration chain exists which includes all models and contains
    /// no loops.
    fn validate_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
    ) -> Result<(), LinearMigratorError> {
        validator::validate_migrations(models)
    }

    /// Migrates data from a starting version to a target version.
    ///
    /// The `LinearMigrator` checks that a migration chain exists between the two given versions,
    /// then iteratively migrates the data through that chain until it is the desired version.
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
                    let next_value = curr_model.migrate(curr_value.as_ref(), next_direction)?;

                    Ok((next_value, next_model))
                },
            )
            .and_then(|(final_value, final_model)| final_model.serialize(final_value.as_ref()));

        debug!(starting_version, target_version, "Migration complete.");

        result
    }

    /// Migrates a given settings value to all other available versions.
    ///
    /// The results from the flood migration include the starting value and version.
    /// Returns an error if one occurs during any migration.
    fn perform_flood_migrations(
        &self,
        models: &dyn ModelStore<ModelKind = Self::ModelKind>,
        starting_value: Box<dyn Any>,
        starting_version: &str,
    ) -> Result<Vec<super::MigrationResult>, Self::ErrorKind> {
        debug!(starting_version, "Starting migrations.");

        let starting_model = models
            .get_model(starting_version)
            .context(error::NoSuchModelSnafu {
                version: starting_version.to_string(),
            })?
            .as_ref();

        let mut results = Vec::with_capacity(models.len());
        results.push(MigrationResult {
            version: starting_model.as_model().get_version(),
            value: starting_model.serialize(starting_value.as_ref())?,
        });

        // Closure which performs all migrations in a direction, pushing results into the result Vec
        let mut flood_migrate = |starting_value: Rc<Box<dyn Any>>, direction| {
            migration_iter(models, starting_version, direction)
                .skip(1)
                .try_fold(
                    (starting_value, starting_model),
                    |(curr_value, curr_model), next_model| {
                        let current_version = curr_model.as_model().get_version();
                        let next_version = next_model.as_model().get_version();
                        debug!(
                            current_version,
                            next_version, "Performing flood submigration."
                        );

                        // Explicitly dereference `Any` pointers to ensure we're downcasting the
                        // right pointer.
                        let unrc_curr_value: &Box<dyn Any> = curr_value.as_ref();
                        let curr_value: &dyn Any = unrc_curr_value.as_ref();
                        let next_value = curr_model.migrate(curr_value, direction)?;

                        results.push(MigrationResult {
                            version: next_version,
                            value: next_model.serialize(next_value.as_ref())?,
                        });

                        Ok((Rc::new(next_value), next_model))
                    },
                )?;
            Ok(())
        };

        let starting_value = Rc::new(starting_value);

        flood_migrate(Rc::clone(&starting_value), Forward)
            .and_then(|_| flood_migrate(starting_value, Backward))?;

        debug!(starting_version, "Flood migration complete.");

        results.sort_by_key(|result| result.version);

        Ok(results)
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

impl MigrationDirection {
    /// Returns the opposite direction to the current.
    fn opposite(self) -> Self {
        match self {
            Forward => Backward,
            Backward => Forward,
        }
    }
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

            migration_iter(all_models, starting_version, direction)
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
}

/// Iterate through the extensions chain of model migrations, starting at a given version.
fn migration_iter<'a>(
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

impl LinearlyMigrateable for NoMigration {
    type ForwardMigrationTarget = NoMigration;
    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget, Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }

    fn migrate_backward(&self) -> Result<Self::ForwardMigrationTarget, Self::ErrorKind> {
        unimplemented!(
            "`NoMigration` used as a marker type. Its settings model should never be used."
        )
    }
}

mod error {
    #![allow(missing_docs)]
    use super::MigrationDirection;
    use snafu::Snafu;

    /// Error type returned by the linear migrator.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum LinearMigratorError {
        #[snafu(display(
            "Detected disjoint migration chains while validating migrations: versions '{}' are not \
            reachable from versions '{}'",
            unreachable_versions.join(", "),
            visited_versions.join(", "),
        ))]
        DisjointMigrationChain {
            unreachable_versions: Vec<String>,
            visited_versions: Vec<String>,
        },

        #[snafu(display("Failed to downcast migrated value as setting version '{}'", version))]
        DowncastSetting { version: &'static str },

        #[snafu(display(
            "Detected an irreversible migration chain: {} points {} to {}, which points {} to {}.",
            lhs_version, direction, fulcrum, direction.opposite(),
            rhs_version.unwrap_or("no migration.")
        ))]
        IrreversibleMigrationChain {
            lhs_version: &'static str,
            fulcrum: &'static str,
            rhs_version: Option<&'static str>,
            direction: MigrationDirection,
        },

        #[snafu(display(
            "Detected a migration loop. Multiple models use version '{}' as a migration target.",
            version
        ))]
        MigrationLoop { version: &'static str },

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
    use crate::BottlerocketSetting;
    use serde::{Deserialize, Serialize};
    use std::convert::Infallible;

    use super::*;

    macro_rules! basic_migrateable {
        ($name:ident, $repr:expr, $backward:ident, $forward:ident) => {
            #[derive(Debug, Serialize, Deserialize)]
            struct $name {
                ident: String,
            }

            impl $name {
                fn new() -> Self {
                    Self {
                        ident: $repr.to_string(),
                    }
                }
            }

            impl crate::SettingsModel for $name {
                type PartialKind = Self;
                type ErrorKind = Infallible;

                fn get_version() -> &'static str {
                    $repr
                }

                fn set(
                    // We allow any transition from current value to target, so we don't need the current value
                    _current_value: Option<Self>,
                    _target: Self,
                ) -> Result<Self, Infallible> {
                    Ok(Self::new())
                }

                fn generate(
                    _existing_partial: Option<Self::PartialKind>,
                    // We do not depend on any settings
                    _dependent_settings: Option<serde_json::Value>,
                ) -> Result<crate::GenerateResult<Self::PartialKind, Self>, Infallible> {
                    Ok(crate::GenerateResult::Complete(Self::new()))
                }

                fn validate(
                    _value: Self,
                    _validated_settings: Option<serde_json::Value>,
                ) -> Result<bool, Infallible> {
                    Ok(true)
                }
            }

            // We have to implement `TypeErasedModel` to make `ModelStore` happy
            impl LinearlyMigrateable for $name {
                type ForwardMigrationTarget = $forward;
                type BackwardMigrationTarget = $backward;

                /// We migrate forward by splitting the motd on whitespace
                fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget, Infallible> {
                    Ok($forward::new())
                }

                fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget, Infallible> {
                    Ok($backward::new())
                }
            }
        };
    }

    basic_migrateable!(BasicV1, "v1", NoMigration, BasicV2);
    basic_migrateable!(BasicV2, "v2", BasicV1, BasicV3);
    basic_migrateable!(BasicV3, "v3", BasicV2, BasicV4);
    basic_migrateable!(BasicV4, "v4", BasicV3, BasicV5);
    basic_migrateable!(BasicV5, "v5", BasicV4, NoMigration);

    fn test_extension_builder() -> LinearMigratorExtensionBuilder {
        LinearMigratorExtensionBuilder::with_name("fake").with_models(vec![
            BottlerocketSetting::<BasicV1>::model(),
            BottlerocketSetting::<BasicV2>::model(),
            BottlerocketSetting::<BasicV3>::model(),
            BottlerocketSetting::<BasicV4>::model(),
            BottlerocketSetting::<BasicV5>::model(),
        ])
    }

    #[test]
    fn test_find_migration_route() {
        let models = test_extension_builder().build().unwrap();

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
        let models = test_extension_builder().build().unwrap();

        let versions = migration_iter(&models, "v1", Forward)
            .map(|model| model.as_model().get_version())
            .collect::<Vec<_>>();

        assert_eq!(versions, vec!["v1", "v2", "v3", "v4", "v5"])
    }

    #[test]
    fn test_target_migration() {
        let models = test_extension_builder().build().unwrap();

        let starting_version = "v1";
        let starting_value = Box::new(BasicV1::new()) as Box<dyn Any>;
        let target_version = "v5";

        assert_eq!(
            LinearMigrator
                .perform_migration(&models, starting_value, starting_version, target_version)
                .unwrap(),
            serde_json::to_value(BasicV5::new()).unwrap()
        );
    }

    #[test]
    fn test_flood_migration() {
        let models = test_extension_builder().build().unwrap();

        let expected_flood_results = vec![
            MigrationResult {
                version: "v1",
                value: serde_json::to_value(BasicV1::new()).unwrap(),
            },
            MigrationResult {
                version: "v2",
                value: serde_json::to_value(BasicV2::new()).unwrap(),
            },
            MigrationResult {
                version: "v3",
                value: serde_json::to_value(BasicV3::new()).unwrap(),
            },
            MigrationResult {
                version: "v4",
                value: serde_json::to_value(BasicV4::new()).unwrap(),
            },
            MigrationResult {
                version: "v5",
                value: serde_json::to_value(BasicV5::new()).unwrap(),
            },
        ];

        vec![
            (Box::new(BasicV1::new()) as Box<dyn Any>, "v1"),
            (Box::new(BasicV2::new()) as Box<dyn Any>, "v2"),
            (Box::new(BasicV3::new()) as Box<dyn Any>, "v3"),
            (Box::new(BasicV4::new()) as Box<dyn Any>, "v4"),
            (Box::new(BasicV5::new()) as Box<dyn Any>, "v5"),
        ]
        .into_iter()
        .for_each(|(starting_value, starting_version)| {
            eprintln!("Testing flood migration starting from {}", starting_version);
            let results = LinearMigrator
                .perform_flood_migrations(&models, starting_value, starting_version)
                .unwrap();
            assert_eq!(results, expected_flood_results)
        });
    }
}
