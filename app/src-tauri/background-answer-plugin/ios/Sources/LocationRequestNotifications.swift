import UserNotifications

/// Owns the LOCATION_REQUEST category and its three actions. Every other notification event
/// goes on to the delegate another plugin set (mobile-push or notification): `inner`.
final class LocationRequestNotifications: NSObject, UNUserNotificationCenterDelegate {
  static let shared = LocationRequestNotifications()
  static let category = "LOCATION_REQUEST"
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
    let ours = UNNotificationCategory(
      identifier: category, actions: actions, intentIdentifiers: [], options: [])
    let center = UNUserNotificationCenter.current()
    center.getNotificationCategories { existing in
      center.setNotificationCategories(existing.filter { $0.identifier != category }.union([ours]))
    }
  }

  func userNotificationCenter(
    _ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
    withCompletionHandler done: @escaping () -> Void
  ) {
    let content = response.notification.request.content
    if content.categoryIdentifier == Self.category,
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
