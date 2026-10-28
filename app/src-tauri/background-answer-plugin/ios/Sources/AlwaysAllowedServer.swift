import Foundation

/// The user's `always_allowed_agents` rows in Supabase, read and written with the user's own
/// session (RLS: agent tokens can never write them). The silent push handler checks here, so
/// the relay alone can never make the phone share without the user's own choice.
enum AlwaysAllowedServer {
  /// The stored agent name ("" when none) if the user always-allowed `clientId`, else nil.
  /// `requesterId` is set for a shared request: only a row for that requester counts.
  static func lookup(_ clientId: String, requesterId: String?, done: @escaping (String?) -> Void) {
    let query = [
      ("select", "client_name"), ("client_id", "eq.\(clientId)"),
      ("requester_user_id", requesterId.map { "eq.\($0)" } ?? "is.null"),
    ]
    request("GET", query: query, body: nil) { data in
      guard let data = data,
        let rows = (try? JSONSerialization.jsonObject(with: data)) as? [[String: Any]],
        let row = rows.first
      else { return done(nil) }
      done(row["client_name"] as? String ?? "")
    }
  }

  /// "Always allow" tapped on a notification: the relay must know before the next request.
  static func add(_ clientId: String, name: String?, requesterId: String? = nil) {
    var body: [String: Any] = ["client_id": clientId, "requester_user_id": requesterId ?? NSNull()]
    // Left out when unknown, so an upsert never blanks a stored name.
    if let name = name, !name.isEmpty { body["client_name"] = name }
    let query = [("on_conflict", "user_id,client_id,requester_user_id")]
    request("POST", query: query, body: body, prefer: "resolution=merge-duplicates,return=minimal") {
      NSLog("[background-answer] always-allowed save %@", $0 == nil ? "failed" : "ok")
    }
  }

  /// `done` gets the body of a 2xx answer, or nil.
  private static func request(
    _ method: String, query: [(String, String)], body: [String: Any]?, prefer: String? = nil,
    done: @escaping (Data?) -> Void
  ) {
    SessionAccessToken.get { token in
      guard let token = token, let settings = RelaySettings.load(),
        var parts = URLComponents(string: "\(settings.supabaseUrl)/rest/v1/always_allowed_agents")
      else { return done(nil) }
      parts.queryItems = query.map { URLQueryItem(name: $0.0, value: $0.1) }
      guard let url = parts.url else { return done(nil) }
      var request = URLRequest(url: url, timeoutInterval: 10)
      request.httpMethod = method
      request.setValue(settings.anonKey, forHTTPHeaderField: "apikey")
      request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
      if let prefer = prefer { request.setValue(prefer, forHTTPHeaderField: "Prefer") }
      if let body = body {
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)
      }
      URLSession.shared.dataTask(with: request) { data, response, _ in
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        done((200..<300).contains(status) ? (data ?? Data()) : nil)
      }.resume()
    }
  }
}
