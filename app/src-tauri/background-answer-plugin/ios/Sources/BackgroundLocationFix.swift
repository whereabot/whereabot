import CoreLocation

/// One location read for one approved request. iOS often reports a coarse cell/Wi-Fi
/// reading first, so watch for up to 10 s and keep the most accurate reading; stop early
/// at street-level accuracy. Create and call on the main thread.
final class BackgroundLocationFix: NSObject, CLLocationManagerDelegate {
  private static let preciseMeters: CLLocationAccuracy = 30
  private static let maxWait: TimeInterval = 10

  private let manager = CLLocationManager()
  private var done: ((CLLocation?) -> Void)?
  private var best: CLLocation?
  private var timer: Timer?

  func read(_ done: @escaping (CLLocation?) -> Void) {
    self.done = done
    manager.delegate = self
    // A precise fix under 30 s old answers at once.
    if let last = manager.location, -last.timestamp.timeIntervalSinceNow < 30,
      last.horizontalAccuracy >= 0, last.horizontalAccuracy <= Self.preciseMeters
    {
      return finish(last)
    }
    manager.desiredAccuracy = kCLLocationAccuracyBest
    manager.startUpdatingLocation()
    timer = Timer.scheduledTimer(withTimeInterval: Self.maxWait, repeats: false) { [weak self] _ in
      self?.finish(self?.best)
    }
  }

  func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
    for location in locations where location.horizontalAccuracy >= 0 {
      if best == nil || location.horizontalAccuracy < best!.horizontalAccuracy { best = location }
    }
    if let best, best.horizontalAccuracy <= Self.preciseMeters { finish(best) }
  }

  func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
    // Transient errors (e.g. kCLErrorLocationUnknown) keep the watch going until the timer.
    if (error as? CLError)?.code == .denied { finish(nil) }
  }

  private func finish(_ location: CLLocation?) {
    timer?.invalidate()
    manager.stopUpdatingLocation()
    let callback = done
    done = nil
    callback?(location)
  }
}
