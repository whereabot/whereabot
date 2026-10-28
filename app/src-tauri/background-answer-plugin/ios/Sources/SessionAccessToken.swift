import Foundation

/// A valid Supabase access token for the relay, refreshed through Supabase when the stored
/// one is about to expire. The rotated session is written back, so the app keeps it too.
enum SessionAccessToken {
  static func get(_ done: @escaping (String?) -> Void) {
    guard let raw = SessionKeychain.read(), let session = parse(Data(raw.utf8)),
      let access = session["access_token"] as? String
    else { return done(nil) }
    let expiresAt = (session["expires_at"] as? Double) ?? 0
    if expiresAt - Date().timeIntervalSince1970 > 60 { return done(access) }
    guard let refresh = session["refresh_token"] as? String, let settings = RelaySettings.load(),
      let url = URL(string: "\(settings.supabaseUrl)/auth/v1/token?grant_type=refresh_token")
    else { return done(nil) }
    var request = URLRequest(url: url, timeoutInterval: 10)
    request.httpMethod = "POST"
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    request.setValue(settings.anonKey, forHTTPHeaderField: "apikey")
    request.httpBody = try? JSONSerialization.data(withJSONObject: ["refresh_token": refresh])
    URLSession.shared.dataTask(with: request) { data, response, _ in
      guard (response as? HTTPURLResponse)?.statusCode == 200, let data = data,
        var fresh = parse(data), let token = fresh["access_token"] as? String
      else {
        NSLog("[background-answer] session refresh failed")
        return done(nil)
      }
      if fresh["expires_at"] == nil, let ttl = fresh["expires_in"] as? Double {
        fresh["expires_at"] = Int(Date().timeIntervalSince1970 + ttl)
      }
      if let json = try? JSONSerialization.data(withJSONObject: fresh),
        let text = String(data: json, encoding: .utf8)
      {
        SessionKeychain.write(text)
      }
      done(token)
    }.resume()
  }

  private static func parse(_ data: Data) -> [String: Any]? {
    (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
  }
}
