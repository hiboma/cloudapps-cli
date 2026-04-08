#![no_main]

use libfuzzer_sys::fuzz_target;

use cloudapps::agent::security::{validate_command_name, CommandWhitelist};

fuzz_target!(|data: &str| {
    // Fuzz command name validation.
    let _ = validate_command_name(data);

    // Fuzz whitelist check with the validated name.
    let wl = CommandWhitelist::default_cloudapps();
    let _ = wl.is_allowed(data);
});
