use bottlerocket_settings_sdk::{BottlerocketSetting, LinearMigratorExtensionBuilder};
use settings_extension_ntp::NtpSettingsV1;
use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();

    // NtpSettingsV1 accepts both time-servers forms through TryFrom, so this
    // extension registers one model. The OS datastore migration is handled separately.
    match LinearMigratorExtensionBuilder::with_name("ntp")
        .with_models(vec![BottlerocketSetting::<NtpSettingsV1>::model()])
        .build()
    {
        Ok(extension) => extension.run(),
        Err(e) => {
            println!("{e}");
            ExitCode::FAILURE
        }
    }
}
