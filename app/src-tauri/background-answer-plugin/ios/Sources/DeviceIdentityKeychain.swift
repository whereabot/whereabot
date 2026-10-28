import Foundation
import Security

/// The install's stable device id. Keychain items outlive an uninstall, so a reinstall keeps
/// the id and the Devices page shows one row. It has its own service: `SessionKeychain`
/// wipes its items on a fresh install and must never touch this one.
enum DeviceIdentityKeychain {
  private static let item: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: "io.whereabot.device-identity",
    kSecAttrAccount as String: "device_id",
    // Not iCloud Keychain: two phones of one user must not share an id.
    kSecAttrSynchronizable as String: false,
  ]

  /// The stored id. Without one, `legacy` (the id the web view kept before) is adopted if it
  /// is a UUID, else a new one is made.
  static func idOrCreate(legacy: String?) -> String {
    if let id = read() { return id }
    let id = legacy.flatMap { UUID(uuidString: $0) }?.uuidString.lowercased()
      ?? UUID().uuidString.lowercased()
    let attributes: [String: Any] = [
      kSecValueData as String: Data(id.utf8),
      kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlock,
    ]
    SecItemAdd(item.merging(attributes) { $1 } as CFDictionary, nil)
    return id
  }

  /// This device was removed from the account: the next sign-in is a new device.
  static func remove() {
    SecItemDelete(item as CFDictionary)
  }

  private static func read() -> String? {
    var query = item
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    var out: AnyObject?
    guard SecItemCopyMatching(query as CFDictionary, &out) == errSecSuccess,
      let data = out as? Data
    else { return nil }
    return String(data: data, encoding: .utf8)
  }
}
