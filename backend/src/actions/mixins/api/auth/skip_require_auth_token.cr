module Api::Auth::SkipRequireAuthToken
  macro included
    # Only skip if the pipe exists (when RequireAuthToken is included in base class)
    {% if @type.has_method?(:require_auth_token) %}
      skip require_auth_token
    {% end %}
  end

  # Since sign in is not required, current_user might be nil
  def current_user : User?
    current_user?
  end
end
