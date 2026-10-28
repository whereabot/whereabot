import Foundation

/// `POST /v1/device/answer` with the phone's Supabase access token. See docs/protocol.md.
enum RelayAnswerPost {
  /// `body` has the same shape as the socket answer. `done` gets the HTTP status, or nil.
  static func send(_ body: [String: Any], done: @escaping (Int?) -> Void) {
    SessionAccessToken.get { token in
      guard let token = token, let settings = RelaySettings.load(),
        let url = URL(string: "\(settings.relayUrl)/v1/device/answer"),
        let json = try? JSONSerialization.data(withJSONObject: body)
      else {
        NSLog("[background-answer] not signed in or not configured")
        return done(nil)
      }
      var request = URLRequest(url: url, timeoutInterval: 10)
      request.httpMethod = "POST"
      request.setValue("application/json", forHTTPHeaderField: "Content-Type")
      request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
      request.httpBody = json
      URLSession.shared.dataTask(with: request) { _, response, _ in
        let status = (response as? HTTPURLResponse)?.statusCode
        NSLog("[background-answer] relay status %d", status ?? -1)
        done(status)
      }.resume()
    }
  }
}
