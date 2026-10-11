module Api::Auth::Helpers
  # The 'memoize' macro makes sure only one query is issued to find the user
  memoize def current_user? : User?
    token = auth_token
    return nil unless token

    user_from_auth_token(token)
  end

  private def auth_token : String?
    bearer_token
  end

  private def bearer_token : String?
    context.request.headers["Authorization"]?
      .try(&.gsub("Bearer ", ""))
      .try(&.strip)
  end

  private def user_from_auth_token(token : String) : User?
    UserToken.user_from_token(token)
  end
end
