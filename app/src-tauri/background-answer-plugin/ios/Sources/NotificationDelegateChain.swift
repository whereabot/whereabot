import ObjectiveC
import UserNotifications

/// Keeps `LocationRequestNotifications` in front of whatever delegate other plugins set.
/// tauri-plugin-mobile-push sets its delegate lazily, on the first token fetch, so a plain
/// assignment at launch would be overwritten. This wraps every later assignment instead.
enum NotificationDelegateChain {
  private static var installed = false

  static func install(_ front: LocationRequestNotifications) {
    guard !installed else { return }
    installed = true
    let cls: AnyClass = UNUserNotificationCenter.self
    guard
      let original = class_getInstanceMethod(cls, #selector(setter: UNUserNotificationCenter.delegate)),
      let wrapper = class_getInstanceMethod(cls, #selector(UNUserNotificationCenter.wb_setDelegate(_:)))
    else {
      NSLog("[background-answer] cannot wrap the notification delegate")
      return
    }
    method_exchangeImplementations(original, wrapper)
    // Wraps the delegate a plugin may already have set.
    let center = UNUserNotificationCenter.current()
    center.delegate = center.delegate
  }
}

extension UNUserNotificationCenter {
  /// After the exchange, `delegate = x` lands here and the call below is the original setter.
  @objc fileprivate func wb_setDelegate(_ delegate: UNUserNotificationCenterDelegate?) {
    let front = LocationRequestNotifications.shared
    if (delegate as AnyObject?) !== front {
      front.inner = delegate
    }
    wb_setDelegate(front)
  }
}
