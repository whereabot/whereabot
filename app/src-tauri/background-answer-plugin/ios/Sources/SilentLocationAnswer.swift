import UIKit

/// Answers a silent locate push (content-available) for an always-allowed agent with no tap,
/// then says so with a local "<Agent> checked your location". See docs/always-allow.md.
enum SilentLocationAnswer {
  /// Main thread. `done` goes back to iOS, which allows about 30 s.
  static func handle(_ userInfo: [AnyHashable: Any], done: @escaping (UIBackgroundFetchResult) -> Void) {
    guard let id = userInfo["request_id"] as? String,
      let clientId = userInfo["client_id"] as? String
    else { return done(.noData) }
    // In the foreground the web view answers over its socket and says so itself.
    if UIApplication.shared.applicationState == .active { return done(.noData) }
    if let expires = userInfo["expires_ms"] as? NSNumber,
      expires.doubleValue < Date().timeIntervalSince1970 * 1000
    {
      return done(.noData)
    }
    // The relay's word is not enough: the user's own row must say "always allowed".
    let requester = RequesterInfo(userInfo)
    AlwaysAllowedServer.lookup(clientId, requesterId: requester?.userId) { stored in
      DispatchQueue.main.async {
        guard let stored = stored else {
          NSLog("[background-answer] silent push for an agent that is not always allowed")
          return done(.noData)
        }
        answer(id, clientId: clientId, name: stored.isEmpty ? (requester?.label(agent: nil) ?? "Your agent") : stored, done: done)
      }
    }
  }

  private static func answer(
    _ id: String, clientId: String, name: String,
    done: @escaping (UIBackgroundFetchResult) -> Void
  ) {
    let fix = BackgroundLocationFix()
    fix.read { location in
      withExtendedLifetime(fix) {}
      RelayAnswerPost.send(LocationRequestAction.answer(id, location)) { status in
        DispatchQueue.main.async {
          // 204 only: a 410 means another device or the web view answered, or time ran out.
          if status == 204, location != nil {
            AutoSharedLog.add(clientId: clientId, name: name)
            CheckedNotification.post(agentName: name, requestId: id)
          }
          done(status == 204 ? .newData : .failed)
        }
      }
    }
  }
}
