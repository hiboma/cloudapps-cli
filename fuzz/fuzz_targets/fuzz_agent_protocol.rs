#![no_main]

use libfuzzer_sys::fuzz_target;

use cloudapps::agent::protocol::{AgentRequest, AgentResponse};

fuzz_target!(|data: &str| {
    // Fuzz AgentRequest JSON deserialization.
    let _ = AgentRequest::from_json_line(data);

    // Fuzz AgentResponse JSON deserialization.
    let _ = AgentResponse::from_json_line(data);
});
