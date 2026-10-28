import Foundation

/// Who asks when a person shares their location with the user: the push carries
/// `requester: {user_id, name, email}` (null for the user's own agent).
struct RequesterInfo {
  let userId: String
  let who: String

  init?(_ userInfo: [AnyHashable: Any]) {
    guard let r = userInfo["requester"] as? [String: Any], let id = r["user_id"] as? String,
      !id.isEmpty
    else { return nil }
    userId = id
    let name = r["name"] as? String ?? ""
    let email = r["email"] as? String ?? ""
    who = !name.isEmpty ? name : (!email.isEmpty ? email : "Someone")
  }

  /// "<Requester>'s agent <Agent>", the words the app shows too (agent-identity.ts).
  func label(agent: String?) -> String {
    guard let agent = agent, !agent.isEmpty else { return "\(who)'s agent" }
    return "\(who)'s agent \(agent)"
  }
}
