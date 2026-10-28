import CoreLocation
import UIKit

/// Answers one locate push from its notification action, without the web view.
enum LocationRequestAction {
  static let reject = "LOCATION_REJECT"
  static let allow = "LOCATION_ALLOW"
  static let always = "LOCATION_ALWAYS"
  /// The one action on the "is checking" fallback for an always-allowed agent; works like Allow.
  static let share = "LOCATION_SHARE"
  static let all: Set<String> = [reject, allow, always, share]

  /// Main thread. Calls `done` once the relay answered, failed, or iOS ends the background time.
  static func handle(_ action: String, userInfo: [AnyHashable: Any], done: @escaping () -> Void) {
    guard let id = userInfo["request_id"] as? String else { return done() }
    let app = UIApplication.shared
    var task = UIBackgroundTaskIdentifier.invalid
    var finished = false
    let finish = {
      DispatchQueue.main.async {
        guard !finished else { return }
        finished = true
        app.endBackgroundTask(task)
        done()
      }
    }
    task = app.beginBackgroundTask(withName: "location-request-answer", expirationHandler: finish)
    if let expires = userInfo["expires_ms"] as? NSNumber,
      expires.doubleValue < Date().timeIntervalSince1970 * 1000
    {
      NSLog("[background-answer] request already expired")
      return finish()
    }
    if action == reject {
      return RelayAnswerPost.send(["type": "denied", "id": id, "reason": "user"]) { _ in finish() }
    }
    if action == always, let clientId = userInfo["client_id"] as? String {
      let agent = userInfo["agent_name"] as? String
      if let requester = RequesterInfo(userInfo) {
        // Only for that requester. Not queued for the web view: it retries own agents only.
        AlwaysAllowedServer.add(
          clientId, name: requester.label(agent: agent), requesterId: requester.userId)
      } else {
        AlwaysAllowedAgents.add(clientId)
        AlwaysAllowedServer.add(clientId, name: agent)
      }
    }
    let fix = BackgroundLocationFix()
    fix.read { location in
      withExtendedLifetime(fix) {}
      RelayAnswerPost.send(answer(id, location)) { _ in finish() }
    }
  }

  /// The relay answer for one read; also used by the silent push handler.
  static func answer(_ id: String, _ location: CLLocation?) -> [String: Any] {
    guard let location = location else {
      // When In Use cannot read in the background: tell the agent it is a permission gap.
      let always = CLLocationManager().authorizationStatus == .authorizedAlways
      return ["type": "unavailable", "id": id, "reason": always ? "no_fix" : "permission"]
    }
    return [
      "type": "location", "id": id,
      "latitude": location.coordinate.latitude,
      "longitude": location.coordinate.longitude,
      "accuracy_m": location.horizontalAccuracy,
      "captured_at_ms": Int64(location.timestamp.timeIntervalSince1970 * 1000),
    ]
  }
}
