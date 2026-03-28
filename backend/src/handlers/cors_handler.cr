class CorsHandler
  include HTTP::Handler

  ALLOWED_ORIGINS = [
    # Production domain - replace with your actual domain
    # "https://your-domain.com",
    "http://localhost:3000",
    "http://localhost:1234",
    "http://127.0.0.1:3000",
  ]

  def call(context)
    origin = context.request.headers["Origin"]?

    if origin && ALLOWED_ORIGINS.includes?(origin)
      context.response.headers["Access-Control-Allow-Origin"] = origin
      context.response.headers["Vary"] = "Origin"
    end

    context.response.headers["Access-Control-Allow-Methods"] = "GET, POST, PUT, PATCH, DELETE, OPTIONS"
    context.response.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization, X-Company-ID"

    if context.request.method == "OPTIONS"
      context.response.status = HTTP::Status::OK
      context.response.content_type = "text/plain"
      context.response.print ""
    else
      call_next(context)
    end
  end
end
