import UserNotifications

/// The local notice after an always-allowed agent got the location with no tap:
/// "<Agent> checked your location". No actions; a tap opens History.
enum CheckedNotification {
  static let category = "LOCATION_CHECKED"

  static func post(agentName: String, requestId: String) {
    let center = UNUserNotificationCenter.current()
    // A "checking" fallback alert for this request may have arrived meanwhile: it is moot now.
    center.getDeliveredNotifications { delivered in
      let stale = delivered.filter {
        $0.request.content.categoryIdentifier != category
          && $0.request.content.userInfo["request_id"] as? String == requestId
      }
      center.removeDeliveredNotifications(withIdentifiers: stale.map { $0.request.identifier })
    }
    let content = UNMutableNotificationContent()
    content.title = "\(agentName) checked your location"
    content.body = "Shared just now. Manage in Whereabot."
    content.categoryIdentifier = category
    content.threadIdentifier = "whereabot-locate"
    content.userInfo = ["request_id": requestId]
    center.add(UNNotificationRequest(identifier: "checked-\(requestId)", content: content, trigger: nil))
  }
}
