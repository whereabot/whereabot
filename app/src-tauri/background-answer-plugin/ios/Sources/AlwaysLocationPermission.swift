import CoreLocation
import UIKit

/// "Always" location access, which a read from a notification action needs. The geolocation
/// plugin only asks for When In Use. Main thread only.
final class AlwaysLocationPermission {
  static let shared = AlwaysLocationPermission()
  private let manager = CLLocationManager()
  private var waiting: [(String) -> Void] = []
  private var promptUp = false

  init() {
    let center = NotificationCenter.default
    // A system prompt resigns the app; becoming active again means it closed.
    center.addObserver(
      forName: UIApplication.willResignActiveNotification, object: nil, queue: .main
    ) { [weak self] _ in self?.promptUp = true }
    center.addObserver(
      forName: UIApplication.didBecomeActiveNotification, object: nil, queue: .main
    ) { [weak self] _ in self?.report() }
  }

  /// "always", "whenInUse", "denied" or "notDetermined".
  func status() -> String {
    switch manager.authorizationStatus {
    case .authorizedAlways: return "always"
    case .authorizedWhenInUse: return "whenInUse"
    case .denied, .restricted: return "denied"
    default: return "notDetermined"
    }
  }

  /// Shows the upgrade prompt (iOS shows it once per install) and reports the status after.
  func request(_ done: @escaping (String) -> Void) {
    waiting.append(done)
    promptUp = false
    manager.requestAlwaysAuthorization()
    // iOS shows nothing when it already asked: report if the app never resigned.
    DispatchQueue.main.asyncAfter(deadline: .now() + 1) { [weak self] in
      if self?.promptUp == false { self?.report() }
    }
  }

  private func report() {
    let callbacks = waiting
    waiting = []
    callbacks.forEach { $0(status()) }
  }
}
