import Foundation

/// Answers the silent push handler sent with no tap, and whether a "checked your location"
/// notification was tapped. The web view takes both (`take_auto_shared`): History gets a
/// "Shared automatically" row per answer, and a tap opens History.
enum AutoSharedLog {
  private static let sharedKey = "whereabouts.native.autoShared"
  private static let openKey = "whereabouts.native.openHistory"

  static func add(clientId: String, name: String) {
    let defaults = UserDefaults.standard
    let items = defaults.array(forKey: sharedKey) ?? []
    let item: [String: Any] = [
      "clientId": clientId, "name": name, "at": Int64(Date().timeIntervalSince1970 * 1000),
    ]
    defaults.set(Array((items + [item]).suffix(50)), forKey: sharedKey)
  }

  static func markOpenHistory() {
    UserDefaults.standard.set(true, forKey: openKey)
  }

  /// Returns `shared` and `openHistory`, and clears both.
  static func take() -> [String: Any] {
    let defaults = UserDefaults.standard
    let out: [String: Any] = [
      "shared": defaults.array(forKey: sharedKey) ?? [],
      "openHistory": defaults.bool(forKey: openKey),
    ]
    defaults.removeObject(forKey: sharedKey)
    defaults.removeObject(forKey: openKey)
    return out
  }
}
