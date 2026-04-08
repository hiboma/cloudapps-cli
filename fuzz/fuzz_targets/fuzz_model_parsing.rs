#![no_main]

use libfuzzer_sys::fuzz_target;

use cloudapps::models::alert::{CloseType, ResolutionStatus, Severity};
use cloudapps::models::data_enrichment::SubnetCategory;
use cloudapps::models::entity::EntityStatus;
use cloudapps::models::file::{FileType, SharingLevel};

fuzz_target!(|data: &str| {
    // Fuzz all from_str_loose enum parsers.
    let _ = Severity::from_str_loose(data);
    let _ = ResolutionStatus::from_str_loose(data);
    let _ = CloseType::from_str_loose(data);
    let _ = EntityStatus::from_str_loose(data);
    let _ = FileType::from_str_loose(data);
    let _ = SharingLevel::from_str_loose(data);
    let _ = SubnetCategory::from_str_loose(data);
});
