module Api::Auth::RequireAuthToken
  macro included
    before require_auth_token
  end

  private def require_auth_token : Lucky::ActionPipes::Continue | Lucky::Response
    # Uses current_user? from Api::Auth::Helpers which properly handles
    # Bearer token extraction and JWT decoding
    unless current_user?
      response.status_code = 401
      return json({error: "Not authenticated"})
    end
    continue
  end
end
