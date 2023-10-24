use super::erased::TypeErasedLinearlyMigrateable;
use super::{
    error, migration_iter, Backward, Forward, LinearMigrator, LinearMigratorError,
    LinearMigratorModel, MigrationDirection, Migrator, ModelStore,
};
use snafu::ensure;
use std::collections::HashSet;
use tracing::debug;

type Result<T> = std::result::Result<T, LinearMigratorError>;

/// Asserts that a single reversible linear migration chain exists which includes all models and
/// contains no loops.
pub(crate) fn validate_migrations(
    models: &dyn ModelStore<ModelKind = LinearMigratorModel>,
) -> Result<()> {
    // Algorithm concept: Start at an arbitrary model, traverse the chain in both directions,
    // While traversing:
    // * Check that our current model points back to the model we arrived from
    // * Check that we have not already seen this model
    // Loop detection: If we see a model multiple times during any iteration then a loop exists.
    // Disjoint set detection: If our list of visited models doesn't contain all models then a
    // disjoint set exists.
    // Reversibility check:  If we find a model which does not point back to the model that pointed
    // to it, the migration chain is not reversible.
    let (starting_version, starting_model) = if let Some(starting_point) = models.iter().next() {
        starting_point
    } else {
        // There are no models
        return Ok(());
    };
    debug!(
        starting_version,
        "Bi-directionally validating linear migration chain from arbitrary starting version."
    );

    let mut visited = validate_in_direction(models, starting_model.as_ref(), Forward)?;
    visited.extend(validate_in_direction(
        models,
        starting_model.as_ref(),
        Backward,
    )?);

    let all_known_models: HashSet<&str> = models
        .iter()
        .map(|(_, model)| model.as_model().get_version())
        .collect();

    debug!("Checking for disjoint migration chains.");
    disjoint_model_check(&all_known_models, &visited)?;

    Ok(())
}

/// Iterates through the models from a starting version, checking that:
/// * We do not visit any model more than once
/// * All forwardlinks have a matching backlink (e.g. A -> B => A <- B)
fn validate_in_direction<'a>(
    models: &'a dyn ModelStore<ModelKind = <LinearMigrator as Migrator>::ModelKind>,
    starting_model: &dyn TypeErasedLinearlyMigrateable,
    direction: MigrationDirection,
) -> Result<HashSet<&'a str>> {
    let starting_version = starting_model.as_model().get_version();
    let mut visited: HashSet<_> = [starting_version].into();

    migration_iter(models, starting_version, direction)
        .skip(1)
        .try_fold(starting_model, |previous_model, curr_model| {
            let version = curr_model.as_model().get_version();
            let previous_version = previous_model.as_model().get_version();

            let opposite = direction.opposite();

            ensure!(
                !visited.contains(version),
                error::MigrationLoopSnafu { version }
            );
            visited.insert(version);

            ensure!(
                curr_model.migrates_to(opposite) == Some(previous_version),
                error::IrreversibleMigrationChainSnafu {
                    lhs_version: previous_version,
                    fulcrum: version,
                    rhs_version: curr_model.migrates_to(opposite),
                    direction,
                },
            );

            Ok(curr_model)
        })?;

    Ok(visited)
}

/// Given the set of all known models and a set of models discovered during a search, returns
/// an error if a disjoint migration chain exists.
fn disjoint_model_check(
    all_known_models: &HashSet<&str>,
    discovered_models: &HashSet<&str>,
) -> Result<()> {
    let disjoint_models: HashSet<&str> = all_known_models
        .symmetric_difference(discovered_models)
        .cloned()
        .collect();

    ensure!(disjoint_models.is_empty(), {
        // Sort versions to make the error message more readable.
        let mut unreachable_versions: Vec<String> =
            disjoint_models.into_iter().map(|s| s.to_string()).collect();
        unreachable_versions.sort();

        let mut visited_versions: Vec<String> =
            discovered_models.iter().map(|s| s.to_string()).collect();
        visited_versions.sort();

        error::DisjointMigrationChainSnafu {
            unreachable_versions,
            visited_versions,
        }
    });

    Ok(())
}
