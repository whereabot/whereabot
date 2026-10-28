import Tauri
import UIKit
import WebKit

struct SessionArgs: Decodable {
  let value: String
}

struct DeviceIdArgs: Decodable {
  let legacy: String?
}

/// Commands for the web view (app/src/background-answer-bridge.ts and
/// app/src/native-session-storage.ts). The notification work starts in `initPlugin`.
@objc(BackgroundAnswerPlugin)
public class BackgroundAnswerPlugin: Plugin {
  @objc public func configure(_ invoke: Invoke) throws {
    try invoke.parseArgs(RelaySettings.self).save()
    invoke.resolve()
  }

  @objc public func saveSession(_ invoke: Invoke) throws {
    SessionKeychain.write(try invoke.parseArgs(SessionArgs.self).value)
    invoke.resolve()
  }

  @objc public func loadSession(_ invoke: Invoke) {
    invoke.resolve(["value": SessionKeychain.read()])
  }

  /// Sign-out or a dead session: the background handler must stop answering for this user.
  @objc public func removeSession(_ invoke: Invoke) {
    SessionKeychain.remove()
    _ = AlwaysAllowedAgents.take()
    invoke.resolve()
  }

  /// The stable device id (app/src/devices/device-identity.ts). `legacy` migrates the web view's id.
  @objc public func deviceId(_ invoke: Invoke) throws {
    let legacy = try invoke.parseArgs(DeviceIdArgs.self).legacy
    invoke.resolve(["id": DeviceIdentityKeychain.idOrCreate(legacy: legacy)])
  }

  @objc public func resetDeviceId(_ invoke: Invoke) {
    DeviceIdentityKeychain.remove()
    invoke.resolve()
  }

  @objc public func takeAlwaysAllowed(_ invoke: Invoke) {
    invoke.resolve(["clientIds": AlwaysAllowedAgents.take()])
  }

  @objc public func locationAuthorization(_ invoke: Invoke) {
    DispatchQueue.main.async {
      invoke.resolve(["status": AlwaysLocationPermission.shared.status()])
    }
  }

  @objc public func requestAlwaysLocation(_ invoke: Invoke) {
    DispatchQueue.main.async {
      AlwaysLocationPermission.shared.request { invoke.resolve(["status": $0]) }
    }
  }
}

@_cdecl("init_plugin_background_answer")
func initPlugin() -> Plugin {
  LocationRequestNotifications.install()
  return BackgroundAnswerPlugin()
}
