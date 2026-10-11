require "socket"

# Service for sending emails via SMTP
# Uses system SMTP settings from SystemSetting
class EmailService
  Log = ::Log.for("email")

  # Send email verification email
  def self.send_verification_email(user : User, verify_url : String) : Bool
    subject = "Потвърдете вашия email - #{SystemSetting.app_name}"

    body = <<-HTML
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #6366f1, #a855f7); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border: 1px solid #e5e7eb; }
        .button { display: inline-block; background: #6366f1; color: white; padding: 14px 30px; text-decoration: none; border-radius: 8px; margin: 20px 0; }
        .footer { text-align: center; color: #6b7280; font-size: 12px; padding: 20px; }
        .info { background: #dbeafe; border: 1px solid #3b82f6; padding: 12px; border-radius: 6px; margin-top: 20px; font-size: 13px; }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="header">
          <h1 style="margin: 0;">#{SystemSetting.app_name}</h1>
        </div>
        <div class="content">
          <h2>Добре дошли#{user.first_name ? ", #{user.first_name}" : ""}!</h2>
          <p>Благодарим ви за регистрацията! Моля, потвърдете вашия email адрес, за да активирате профила си.</p>
          <p style="text-align: center;">
            <a href="#{verify_url}" class="button">Потвърди email</a>
          </p>
          <p>Или копирайте този линк в браузъра:</p>
          <p style="word-break: break-all; background: #e5e7eb; padding: 10px; border-radius: 4px; font-size: 13px;">
            #{verify_url}
          </p>
          <div class="info">
            <strong>ℹ️ Забележка:</strong> Този линк е валиден 24 часа. Ако не сте заявили регистрация, игнорирайте този email.
          </div>
        </div>
        <div class="footer">
          <p>© #{Time.utc.year} #{SystemSetting.app_name}. Всички права запазени.</p>
        </div>
      </div>
    </body>
    </html>
    HTML

    send_email(user.email, subject, body, html: true)
  end

  # Send password reset email
  def self.send_password_reset(user : User, reset_url : String) : Bool
    subject = "Възстановяване на парола - #{SystemSetting.app_name}"

    body = <<-HTML
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #6366f1, #a855f7); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border: 1px solid #e5e7eb; }
        .button { display: inline-block; background: #6366f1; color: white; padding: 14px 30px; text-decoration: none; border-radius: 8px; margin: 20px 0; }
        .footer { text-align: center; color: #6b7280; font-size: 12px; padding: 20px; }
        .warning { background: #fef3c7; border: 1px solid #f59e0b; padding: 12px; border-radius: 6px; margin-top: 20px; font-size: 13px; }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="header">
          <h1 style="margin: 0;">#{SystemSetting.app_name}</h1>
        </div>
        <div class="content">
          <h2>Здравейте#{user.first_name ? ", #{user.first_name}" : ""}!</h2>
          <p>Получихме заявка за възстановяване на паролата за вашия акаунт.</p>
          <p>Натиснете бутона по-долу, за да зададете нова парола:</p>
          <p style="text-align: center;">
            <a href="#{reset_url}" class="button">Възстанови паролата</a>
          </p>
          <p>Или копирайте този линк в браузъра:</p>
          <p style="word-break: break-all; background: #e5e7eb; padding: 10px; border-radius: 4px; font-size: 13px;">
            #{reset_url}
          </p>
          <div class="warning">
            <strong>⚠️ Важно:</strong> Този линк е валиден само 1 час. Ако не сте заявили възстановяване на парола, игнорирайте този email.
          </div>
        </div>
        <div class="footer">
          <p>© #{Time.utc.year} #{SystemSetting.app_name}. Всички права запазени.</p>
        </div>
      </div>
    </body>
    </html>
    HTML

    send_email(user.email, subject, body, html: true)
  end

  # Send test email to verify SMTP settings
  def self.send_test_email(to : String) : Bool
    subject = "Тестов email - #{SystemSetting.app_name}"

    body = <<-HTML
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #27ae60, #2ecc71); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border: 1px solid #e5e7eb; }
        .success { background: #d1fae5; border: 1px solid #10b981; padding: 15px; border-radius: 6px; text-align: center; }
        .footer { text-align: center; color: #6b7280; font-size: 12px; padding: 20px; }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="header">
          <h1 style="margin: 0;">#{SystemSetting.app_name}</h1>
        </div>
        <div class="content">
          <div class="success">
            <h2 style="color: #059669; margin: 0 0 10px 0;">✓ SMTP работи!</h2>
            <p style="margin: 0;">Тестовият email беше изпратен успешно.</p>
          </div>
          <p style="margin-top: 20px; text-align: center; color: #6b7280;">
            Изпратен на: #{Time.utc.to_s("%Y-%m-%d %H:%M:%S UTC")}
          </p>
        </div>
        <div class="footer">
          <p>© #{Time.utc.year} #{SystemSetting.app_name}. Всички права запазени.</p>
        </div>
      </div>
    </body>
    </html>
    HTML

    send_email(to, subject, body, html: true)
  end

  # Send test email with custom SMTP settings (for testing before save)
  def self.send_test_email_with_settings(
    to : String,
    host : String,
    port : Int32,
    username : String,
    password : String,
    from_email : String,
    from_name : String,
    use_tls : Bool
  ) : NamedTuple(success: Bool, message: String)
    subject = "Тестов email - SMTP конфигурация"

    body = <<-HTML
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #27ae60, #2ecc71); color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border: 1px solid #e5e7eb; }
        .success { background: #d1fae5; border: 1px solid #10b981; padding: 15px; border-radius: 6px; text-align: center; }
        .footer { text-align: center; color: #6b7280; font-size: 12px; padding: 20px; }
      </style>
    </head>
    <body>
      <div class="container">
        <div class="header">
          <h1 style="margin: 0;">Baraba</h1>
        </div>
        <div class="content">
          <div class="success">
            <h2 style="color: #059669; margin: 0 0 10px 0;">✓ SMTP работи!</h2>
            <p style="margin: 0;">Вашите SMTP настройки са конфигурирани правилно.</p>
          </div>
          <p style="margin-top: 20px; text-align: center; color: #6b7280;">
            Сървър: #{host}:#{port}<br>
            Изпратен на: #{Time.utc.to_s("%Y-%m-%d %H:%M:%S UTC")}
          </p>
        </div>
        <div class="footer">
          <p>© #{Time.utc.year} Baraba</p>
        </div>
      </div>
    </body>
    </html>
    HTML

    begin
      send_via_smtp(
        host: host,
        port: port,
        username: username,
        password: password,
        from_email: from_email.empty? ? username : from_email,
        from_name: from_name.empty? ? "Baraba" : from_name,
        to: to,
        subject: subject,
        body: body,
        html: true,
        use_tls: use_tls
      )
      Log.info { "Test email sent to #{to}" }
      {success: true, message: "Тестов email изпратен успешно до #{to}"}
    rescue ex
      Log.error { "Failed to send test email: #{ex.message}" }
      {success: false, message: "Грешка: #{ex.message}"}
    end
  end

  # Send a generic email
  def self.send_email(to : String, subject : String, body : String, html : Bool = false) : Bool
    unless SystemSetting.smtp_enabled?
      Log.warn { "SMTP is not enabled. Email not sent to #{to}" }
      return false
    end

    host = SystemSetting.smtp_host
    port = SystemSetting.smtp_port
    username = SystemSetting.smtp_username
    password = SystemSetting.smtp_password
    from_email = SystemSetting.smtp_from_email || username
    from_name = SystemSetting.smtp_from_name || SystemSetting.app_name
    use_tls = SystemSetting.smtp_use_tls?

    if host.nil? || username.nil? || password.nil?
      Log.error { "SMTP not configured properly" }
      return false
    end

    begin
      send_via_smtp(
        host: host.not_nil!,
        port: port,
        username: username.not_nil!,
        password: password.not_nil!,
        from_email: from_email.not_nil!,
        from_name: from_name.not_nil!,
        to: to,
        subject: subject,
        body: body,
        html: html,
        use_tls: use_tls
      )
      Log.info { "Email sent to #{to}: #{subject}" }
      true
    rescue ex
      Log.error { "Failed to send email to #{to}: #{ex.message}" }
      false
    end
  end

  private def self.send_via_smtp(
    host : String,
    port : Int32,
    username : String,
    password : String,
    from_email : String,
    from_name : String,
    to : String,
    subject : String,
    body : String,
    html : Bool,
    use_tls : Bool
  ) : Nil
    Log.info { "Connecting to SMTP server #{host}:#{port} (TLS: #{use_tls})" }

    socket = TCPSocket.new(host, port)
    socket.read_timeout = 30.seconds
    socket.write_timeout = 30.seconds

    io : IO = socket

    # Port 465 uses implicit TLS (direct SSL connection)
    # Port 587, 25, 80 use plain connection first
    if port == 465
      Log.info { "Using implicit TLS (port 465)" }
      context = OpenSSL::SSL::Context::Client.new
      context.verify_mode = OpenSSL::SSL::VerifyMode::NONE
      ssl_socket = OpenSSL::SSL::Socket::Client.new(socket, context, hostname: host)
      io = ssl_socket
    end

    # Read greeting
    greeting = read_response(io)
    Log.debug { "SMTP greeting: #{greeting}" }
    unless greeting.starts_with?("220")
      raise "SMTP server rejected connection: #{greeting}"
    end

    # EHLO
    send_command(io, "EHLO localhost")

    # Start TLS only if:
    # - Not port 465 (already using implicit TLS)
    # - Not port 80 (Alibaba DirectMail on port 80 doesn't use TLS)
    # - TLS is enabled
    if port != 465 && port != 80 && use_tls
      Log.info { "Attempting STARTTLS" }
      smtp_send(io, "STARTTLS")
      response = read_response(io)
      Log.debug { "STARTTLS response: #{response}" }
      if response.starts_with?("220")
        context = OpenSSL::SSL::Context::Client.new
        context.verify_mode = OpenSSL::SSL::VerifyMode::NONE
        ssl_socket = OpenSSL::SSL::Socket::Client.new(socket, context, hostname: host)
        io = ssl_socket
        send_command(io, "EHLO localhost")
      else
        Log.warn { "STARTTLS not supported: #{response}" }
      end
    end

    # Try AUTH LOGIN first, then AUTH PLAIN if it fails
    smtp_send(io, "AUTH LOGIN")
    auth_response = read_response(io)
    Log.debug { "AUTH LOGIN response: #{auth_response}" }

    if auth_response.starts_with?("334")
      # AUTH LOGIN flow
      smtp_send(io, Base64.strict_encode(username))
      read_response(io)

      smtp_send(io, Base64.strict_encode(password))
      auth_result = read_response(io)
      Log.debug { "AUTH result: #{auth_result}" }
      unless auth_result.starts_with?("235")
        raise "SMTP authentication failed: #{auth_result}"
      end
    elsif auth_response.includes?("AUTH PLAIN") || auth_response.starts_with?("5")
      # Try AUTH PLAIN
      Log.info { "Trying AUTH PLAIN" }
      auth_string = Base64.strict_encode("\0#{username}\0#{password}")
      smtp_send(io, "AUTH PLAIN #{auth_string}")
      auth_result = read_response(io)
      Log.debug { "AUTH PLAIN result: #{auth_result}" }
      unless auth_result.starts_with?("235")
        raise "SMTP authentication failed: #{auth_result}"
      end
    else
      raise "SMTP server does not support authentication: #{auth_response}"
    end

    # MAIL FROM
    send_command(io, "MAIL FROM:<#{from_email}>")

    # RCPT TO
    send_command(io, "RCPT TO:<#{to}>")

    # DATA
    smtp_send(io, "DATA")
    data_response = read_response(io)
    Log.debug { "DATA response: #{data_response}" }
    unless data_response.starts_with?("354")
      raise "SMTP DATA rejected: #{data_response}"
    end

    # Email headers and body - must use CRLF
    content_type = html ? "text/html; charset=UTF-8" : "text/plain; charset=UTF-8"
    message_id = "#{Random::Secure.hex(16)}@#{host}"

    # Build message with proper CRLF line endings
    message = String.build do |msg|
      msg << "From: =?UTF-8?B?#{Base64.strict_encode(from_name)}?= <#{from_email}>\r\n"
      msg << "To: #{to}\r\n"
      msg << "Subject: =?UTF-8?B?#{Base64.strict_encode(subject)}?=\r\n"
      msg << "MIME-Version: 1.0\r\n"
      msg << "Content-Type: #{content_type}\r\n"
      msg << "Content-Transfer-Encoding: base64\r\n"
      msg << "Message-ID: <#{message_id}>\r\n"
      msg << "Date: #{Time.utc.to_rfc2822}\r\n"
      msg << "\r\n"
      # Split base64 body into 76 char lines per RFC 2045
      encoded_body = Base64.strict_encode(body)
      encoded_body.each_char.each_slice(76) do |chunk|
        msg << chunk.join
        msg << "\r\n"
      end
    end

    io << message
    smtp_send(io, ".")  # End of message
    send_response = read_response(io)
    Log.debug { "Message sent response: #{send_response}" }
    unless send_response.starts_with?("250")
      raise "SMTP message rejected: #{send_response}"
    end

    # QUIT
    smtp_send(io, "QUIT")
    read_response(io)

    io.close
    socket.close
    Log.info { "Email sent successfully to #{to}" }
  end

  # Send SMTP command with proper CRLF
  private def self.smtp_send(io : IO, data : String)
    io << data
    io << "\r\n"
    io.flush
  end

  private def self.send_command(io : IO, command : String)
    smtp_send(io, command)
    response = read_response(io)
    Log.debug { "#{command.split.first}: #{response}" }
    unless response.starts_with?("2") || response.starts_with?("3")
      raise "SMTP command failed: #{command} -> #{response}"
    end
    response
  end

  private def self.read_response(io : IO) : String
    response = ""
    loop do
      line = io.gets
      break if line.nil?
      response += line + "\n"
      # Multi-line responses have - after code, last line has space
      break unless line.size >= 4 && line[3] == '-'
    end
    response.strip
  end
end
