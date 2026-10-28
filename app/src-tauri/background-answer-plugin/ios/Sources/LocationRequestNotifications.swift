import UserNotifications

/// Owns the LOCATION_REQUEST category (three actions) and LOCATION_AUTO (Share), and notes taps
/// on LOCATION_CHECKED. Every other notification event goes on to the delegate another plugin
/// set (mobile-push or notification): `inner`.
final class LocationRequestNotifications: NSObject, UNUserNotificationCenterDelegate {
  static let shared = LocationRequestNotifications()
  static let category = "LOCATION_REQUEST"
  /// Fallback alert for an always-allowed agent: "<Agent> is checking your location".
  static let autoCategory = "LOCATION_AUTO"
  var inner: UNUserNotificationCenterDelegate?

  /// Call at launch, before iOS hands over the action that launched the app.
  static func install() {
    registerCategory()
    NotificationDelegateChain.install(shared)
  }

  private static func registerCategory() {
    // No .foreground option: iOS runs these in the background and leaves the app closed.
    let actions = [
      UNNotificationAction(
        identifier: LocationRequestAction.reject, title: "Reject", options: [.destructive]),
      UNNotificationAction(identifier: LocationRequestAction.allow, title: "Allow", options: []),
      UNNotificationAction(
        identifier: LocationRequestAction.always, title: "Always allow", options: []),
    ]
    let share = UNNotificationAction(
      identifier: LocationRequestAction.share, title: "Share", options: [])
    let ours: Set = [
      UNNotificationCategory(
        identifier: category, actions: actions, intentIdentifiers: [], options: []),
      UNNotificationCategory(
        identifier: autoCategory, actions: [share], intentIdentifiers: [], options: []),
    ]
    let ids = Set(ours.map(\.identifier))
    let center = UNUserNotificationCenter.current()
    center.getNotificationCategories { existing in
      center.setNotificationCategories(existing.filter { !ids.contains($0.identifier) }.union(ours))
    }
  }

  func userNotificationCenter(
    _ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
    withCompletionHandler done: @escaping () -> Void
  ) {
    let content = response.notification.request.content
    if content.categoryIdentifier == CheckedNotification.category {
      AutoSharedLog.markOpenHistory()
    }
    if [Self.category, Self.autoCategory].contains(content.categoryIdentifier),
      LocationRequestAction.all.contains(response.actionIdentifier)
    {
      let userInfo = content.userInfo
      let action = response.actionIdentifier
      DispatchQueue.main.async {
        LocationRequestAction.handle(action, userInfo: userInfo, done: done)
      }
      return
    }
    if inner?.userNotificationCenter?(center, didReceive: response, withCompletionHandler: done)
      == nil
    {
      done()
    }
  }

  func userNotificationCenter(
    _ center: UNUserNotificationCenter, willPresent notification: UNNotification,
    withCompletionHandler done: @escaping (UNNotificationPresentationOptions) -> Void
  ) {
    if inner?.userNotificationCenter?(
      center, willPresent: notification, withCompletionHandler: done) == nil
    {
      done([.banner, .list, .sound])
    }
  }
}
