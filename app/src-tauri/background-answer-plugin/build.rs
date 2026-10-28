// Every command runs in Swift (ios/Sources). Off iOS, invokes fail and the JS side falls back.
const COMMANDS: &[&str] = &[
    "configure",
    "save_session",
    "load_session",
    "remove_session",
    "take_always_allowed",
    "location_authorization",
    "request_always_location",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
