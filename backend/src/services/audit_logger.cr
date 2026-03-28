module AuditLogger
  Log = ::Log.for("audit")

  def self.log(context : HTTP::Server::Context, event : String, email : String, details : String? = nil)
    ip = context.request.headers["X-Forwarded-For"]?.try(&.split(",").first.strip)
    ip ||= if addr = context.request.remote_address
             addr.is_a?(Socket::IPAddress) ? addr.address : addr.to_s
           else
             "unknown"
           end

    ua = context.request.headers["User-Agent"]?

    begin
      SaveAuditLog.create!(
        event: event,
        email: email,
        ip_address: ip,
        user_agent: ua,
        details: details
      )
    rescue ex
      Log.warn { "Failed to save audit log: #{ex.message}" }
    end

    Log.info { "[#{event}] email=#{email} ip=#{ip}" }
  end
end
