//! Prints the MCP tool list and the wire message schemas as JSON. The docs build reads it.
use schemars::schema_for;
use whereabouts_server::{FromDevice, ToDevice, tool_list_json};

fn main() {
    let reference = serde_json::json!({
        "mcp_tools": tool_list_json(),
        "to_device": schema_for!(ToDevice),
        "from_device": schema_for!(FromDevice),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&reference).expect("reference serializes")
    );
}
