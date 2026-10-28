import Foundation

/// Where the background handler posts: set by the web view on every launch (`configure`).
/// No secrets: the anon key is public.
struct RelaySettings: Codable {
  /// HTTP(S) base of the relay, e.g. `https://relay.whereabot.io`.
  let relayUrl: String
  let supabaseUrl: String
  let anonKey: String

  private static let key = "whereabouts.native.settings"

  func save() {
    UserDefaults.standard.set(try? JSONEncoder().encode(self), forKey: Self.key)
  }

  static func load() -> RelaySettings? {
    guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
    return try? JSONDecoder().decode(RelaySettings.self, from: data)
  }
}
