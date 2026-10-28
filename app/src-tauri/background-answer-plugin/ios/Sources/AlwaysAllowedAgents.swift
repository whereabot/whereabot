import Foundation

/// Agents the user chose "Always allow" for from a notification. They also go to Supabase at
/// once (AlwaysAllowedServer); this queue lets the web view retry on its next launch.
enum AlwaysAllowedAgents {
  private static let key = "whereabouts.native.alwaysAllowed"
  /// Ids shared by many agents (app/src/agent-identity.ts): never always-allowed.
  private static let generic: Set<String> = ["unknown", "agent-key"]

  static func add(_ clientId: String) {
    guard !generic.contains(clientId) else { return }
    let ids = UserDefaults.standard.stringArray(forKey: key) ?? []
    UserDefaults.standard.set(ids.filter { $0 != clientId } + [clientId], forKey: key)
  }

  /// Returns the pending ids and clears them.
  static func take() -> [String] {
    let ids = UserDefaults.standard.stringArray(forKey: key) ?? []
    UserDefaults.standard.removeObject(forKey: key)
    return ids
  }
}
