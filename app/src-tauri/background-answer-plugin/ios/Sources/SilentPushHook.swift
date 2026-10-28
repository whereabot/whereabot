import ObjectiveC
import UIKit

/// iOS hands a silent push to the app delegate, which tao (Tauri) owns. This adds
/// `application:didReceiveRemoteNotification:fetchCompletionHandler:` to its class, the way
/// tauri-plugin-mobile-push adds its token callbacks. Needs UIBackgroundModes remote-notification.
enum SilentPushHook {
  private static var installed = false
  private typealias Completion = @convention(block) (UIBackgroundFetchResult) -> Void
  private typealias Handler =
    @convention(block) (AnyObject, UIApplication, NSDictionary, @escaping Completion) -> Void

  /// Main thread. Retries for a few seconds until tao has set the app delegate.
  static func install(attempts: Int = 50) {
    guard !installed else { return }
    guard let delegate = UIApplication.shared.delegate else {
      if attempts > 0 {
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { install(attempts: attempts - 1) }
      }
      return
    }
    installed = true
    let selector = sel_registerName("application:didReceiveRemoteNotification:fetchCompletionHandler:")
    let handler: Handler = { _, _, userInfo, done in
      let info = userInfo as? [AnyHashable: Any] ?? [:]
      // Only our silent locate push: alerts go through the notification delegate.
      let aps = info["aps"] as? [String: Any]
      guard aps?["alert"] == nil, info["request_id"] != nil else { return done(.noData) }
      DispatchQueue.main.async { SilentLocationAnswer.handle(info, done: done) }
    }
    if !class_addMethod(
      type(of: delegate), selector, imp_implementationWithBlock(handler), "v@:@@@?")
    {
      NSLog("[background-answer] the app delegate already handles remote notifications")
    }
  }
}
