class Errors::Show < Lucky::ErrorAction
  DEFAULT_MESSAGE = "Something went wrong."
  default_format :json

  dont_report [Lucky::RouteNotFoundError, Avram::RecordNotFoundError]

  def render(error : Lucky::RouteNotFoundError)
    if json?
      json({success: false, error: "Not found"}, status: 404)
    else
      html_error_page("Not found", status: 404)
    end
  end

  def render(error : Avram::RecordNotFoundError)
    if json?
      json({success: false, error: "Record not found"}, status: 404)
    else
      html_error_page("Record not found", status: 404)
    end
  end

  def render(error : Lucky::InvalidParamError)
    if json?
      json({success: false, error: error.message}, status: 422)
    else
      html_error_page("Invalid parameter", status: 422)
    end
  end

  def default_render(error : Exception) : Lucky::Response
    if json?
      # Show actual error in development
      error_msg = LuckyEnv.development? ? "#{error.class}: #{error.message}" : DEFAULT_MESSAGE
      json({success: false, error: error_msg}, status: 500)
    else
      html_error_page(DEFAULT_MESSAGE, status: 500)
    end
  end

  def report(error : Exception) : Nil
    Lucky::Log.error(exception: error) { "" }
  end

  private def html_error_page(message : String, status : Int32) : Lucky::Response
    Lucky::TextResponse.new(context, content_type: "text/html", body: "<html><body><h1>#{message}</h1></body></html>", status: status)
  end

  private def json?
    context.request.headers["Accept"]?.try(&.includes?("application/json")) ||
      context.request.path.starts_with?("/api")
  end
end
