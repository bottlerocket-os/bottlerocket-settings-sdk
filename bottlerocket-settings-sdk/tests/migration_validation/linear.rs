use anyhow::Result;
use bottlerocket_settings_sdk::{
    extension::SettingsExtensionError, BottlerocketSetting, GenerateResult,
    LinearMigratorExtensionBuilder, LinearlyMigrateable, NoMigration, SettingsModel,
};
use serde::{Deserialize, Serialize};

use super::*;

macro_rules! define_model {
    ($name:ident, $version:expr, $forward:ident, $backward:ident) => {
        common::define_model!($name, $version);

        impl LinearlyMigrateable for $name {
            type ForwardMigrationTarget = $forward;
            type BackwardMigrationTarget = $backward;

            fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
                unimplemented!()
            }

            fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
                unimplemented!()
            }
        }
    };
}

define_model!(DisjointA, "v1", NoMigration, NoMigration);
define_model!(DisjointB, "v2", NoMigration, NoMigration);

#[test]
fn test_no_small_disjoint_islands() {
    // Given two models which do not link in a migration chain,
    // When an linear migrator extension is built with those models,
    // The extension will fail to build.

    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("disjoint-models")
            .with_models(vec![
                BottlerocketSetting::<DisjointA>::model(),
                BottlerocketSetting::<DisjointB>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// A <-> B <-> D
// E <-> C <-> A
define_model!(LargeDisjointA, "v1", LargeDisjointB, NoMigration);
define_model!(LargeDisjointB, "v2", LargeDisjointD, LargeDisjointA);
define_model!(LargeDisjointC, "v3", LargeDisjointA, LargeDisjointE);
define_model!(LargeDisjointD, "v4", NoMigration, LargeDisjointB);
define_model!(LargeDisjointE, "v5", NoMigration, LargeDisjointC);

#[test]
fn test_no_large_disjoint_islands() {
    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("disjoint-models")
            .with_models(vec![
                BottlerocketSetting::<LargeDisjointA>::model(),
                BottlerocketSetting::<LargeDisjointB>::model(),
                BottlerocketSetting::<LargeDisjointC>::model(),
                BottlerocketSetting::<LargeDisjointD>::model(),
                BottlerocketSetting::<LargeDisjointE>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// A <-> C <-> D
// B ---^
define_model!(DoubleTailedA, "v1", DoubleTailedC, NoMigration);
define_model!(DoubleTailedB, "v2", DoubleTailedC, NoMigration);
define_model!(DoubleTailedC, "v3", DoubleTailedD, DoubleTailedA);
define_model!(DoubleTailedD, "v4", NoMigration, DoubleTailedC);

#[test]
fn test_no_double_tail() {
    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("disjoint-models")
            .with_models(vec![
                BottlerocketSetting::<DoubleTailedA>::model(),
                BottlerocketSetting::<DoubleTailedB>::model(),
                BottlerocketSetting::<DoubleTailedC>::model(),
                BottlerocketSetting::<DoubleTailedD>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// C <-> A <-> B <-> C
define_model!(LoopA, "v1", LoopC, LoopB);
define_model!(LoopB, "v2", LoopA, LoopC);
define_model!(LoopC, "v3", LoopB, LoopA);

#[test]
fn test_no_migration_loops_simple_circle() {
    // Given a simple loop of linear migrations between models,
    // When an linear migrator extension is built with those models,
    // The extension will fail to build.

    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("circular-loop")
            .with_models(vec![
                BottlerocketSetting::<LoopA>::model(),
                BottlerocketSetting::<LoopB>::model(),
                BottlerocketSetting::<LoopC>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// A <-> B -> C
// ^----------|
define_model!(BrokenBacklinkA, "v1", NoMigration, LoopB);
define_model!(BrokenBacklinkB, "v2", LoopA, LoopC);
// C mistakenly points back to A
define_model!(BrokenBacklinkC, "v3", LoopA, NoMigration);

#[test]
fn test_no_migration_loops_backlink() {
    // Given a set of models with a backwards migration resulting in a loop,
    // When an linear migrator extension is built with those models,
    // The extension will fail to build.

    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("broken-backlink")
            .with_models(vec![
                BottlerocketSetting::<BrokenBacklinkA>::model(),
                BottlerocketSetting::<BrokenBacklinkB>::model(),
                BottlerocketSetting::<BrokenBacklinkC>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// A mistakenly points back to C
define_model!(BackwardsCycleA, "v1", BackwardsCycleC, BackwardsCycleB);
define_model!(BackwardsCycleB, "v2", BackwardsCycleA, BackwardsCycleC);
define_model!(BackwardsCycleC, "v3", BackwardsCycleB, NoMigration);

#[test]
fn test_no_migration_loops_backcycle() {
    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("backcycle")
            .with_models(vec![
                BottlerocketSetting::<BackwardsCycleA>::model(),
                BottlerocketSetting::<BackwardsCycleB>::model(),
                BottlerocketSetting::<BackwardsCycleC>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

define_model!(ForwardsCycleA, "v1", NoMigration, ForwardsCycleB);
define_model!(ForwardsCycleB, "v2", ForwardsCycleA, ForwardsCycleC);
// C mistakenly points forward to A
define_model!(ForwardsCycleC, "v3", ForwardsCycleB, ForwardsCycleA);

#[test]
fn test_no_migration_loops_forwardcycle() {
    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("forwards-cycle")
            .with_models(vec![
                BottlerocketSetting::<ForwardsCycleA>::model(),
                BottlerocketSetting::<ForwardsCycleB>::model(),
                BottlerocketSetting::<ForwardsCycleC>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}

// A -> B -> C -> D
// A <- C <- B <- D
define_model!(NotReversibleA, "v1", NotReversibleB, NoMigration);
define_model!(NotReversibleB, "v2", NotReversibleC, NotReversibleC);
define_model!(NotReversibleC, "v3", NotReversibleD, NotReversibleA);
define_model!(NotReversibleD, "v4", NoMigration, NotReversibleB);

#[test]
fn test_no_non_reversible() {
    assert!(matches!(
        LinearMigratorExtensionBuilder::with_name("not-reversible")
            .with_models(vec![
                BottlerocketSetting::<NotReversibleA>::model(),
                BottlerocketSetting::<NotReversibleB>::model(),
                BottlerocketSetting::<NotReversibleC>::model(),
                BottlerocketSetting::<NotReversibleD>::model(),
            ])
            .build(),
        Err(SettingsExtensionError::MigrationValidation { .. })
    ));
}
