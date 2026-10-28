import CoreLocation

/// One location read for one approved request: best accuracy for up to 8 s, then
/// hundred-meter accuracy for up to 8 s more. Create and call on the main thread.
final class BackgroundLocationFix: NSObject, CLLocationManagerDelegate {
  private let manager = CLLocationManager()
  private var done: ((CLLocation?) -> Void)?
  private var fellBack = false
  private var timer: Timer?

  func read(_ done: @escaping (CLLocation?) -> Void) {
    self.done = done
    manager.delegate = self
    // A fix up to 30 s old answers at once, as in the app.
    if let last = manager.location, -last.timestamp.timeIntervalSinceNow < 30 {
      return finish(last)
    }
    ask(kCLLocationAccuracyBest)
  }

  func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
    finish(locations.last)
  }

  func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
    fallBack()
  }

  private func ask(_ accuracy: CLLocationAccuracy) {
    timer?.invalidate()
    manager.stopUpdatingLocation()
    manager.desiredAccuracy = accuracy
    manager.requestLocation()
    timer = Timer.scheduledTimer(withTimeInterval: 8, repeats: false) { [weak self] _ in
      self?.fallBack()
    }
  }

  private func fallBack() {
    guard done != nil else { return }
    if fellBack { return finish(nil) }
    fellBack = true
    ask(kCLLocationAccuracyHundredMeters)
  }

  private func finish(_ location: CLLocation?) {
    timer?.invalidate()
    manager.stopUpdatingLocation()
    let callback = done
    done = nil
    callback?(location)
  }
}
