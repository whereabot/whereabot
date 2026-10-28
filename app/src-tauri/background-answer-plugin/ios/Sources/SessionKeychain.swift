import Foundation
import Security

/// The Supabase session, as the JSON string supabase-js stores. The web view reads and writes
/// it through this item (app/src/native-session-storage.ts), so the background handler and
/// the app share one refresh token. Never logged.
enum SessionKeychain {
  private static let item: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: "io.whereabot.session",
    kSecAttrAccount as String: "supabase",
  ]
  private static let installedKey = "whereabouts.native.keychainInstalled"

  static func read() -> String? {
    dropAfterReinstall()
    var query = item
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    var out: AnyObject?
    guard SecItemCopyMatching(query as CFDictionary, &out) == errSecSuccess,
      let data = out as? Data
    else { return nil }
    return String(data: data, encoding: .utf8)
  }

  static func write(_ value: String) {
    dropAfterReinstall()
    // Readable after the first unlock, so a background launch from a push can read it.
    let attributes: [String: Any] = [
      kSecValueData as String: Data(value.utf8),
      kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlock,
    ]
    if SecItemUpdate(item as CFDictionary, attributes as CFDictionary) == errSecItemNotFound {
      SecItemAdd(item.merging(attributes) { $1 } as CFDictionary, nil)
    }
  }

  static func remove() {
    SecItemDelete(item as CFDictionary)
  }

  /// Keychain items outlive an uninstall; a fresh install must not sign in as the old user.
  private static func dropAfterReinstall() {
    guard !UserDefaults.standard.bool(forKey: installedKey) else { return }
    remove()
    UserDefaults.standard.set(true, forKey: installedKey)
  }
}
