# In-memory tracking of failed login attempts for account lockout.
# Uses the "security" system settings: max_login_attempts, lockout_duration_minutes.
class LoginAttemptTracker
  Log = ::Log.for("auth.lockout")

  MAX_TRACKED_ACCOUNTS = 10_000

  @@attempts = {} of String => {count: Int32, locked_until: Time?}
  @@mutex = Mutex.new

  def self.lock_message(key : String) : String?
    until_time = locked_until(key.downcase)
    return nil unless until_time

    minutes = ((until_time - Time.utc).total_minutes).ceil.to_i
    minutes = 1 if minutes < 1
    "Твърде много неуспешни опити. Акаунтът е заключен за още #{minutes} минути."
  end

  # Returns the time until which the account is locked, or nil if not locked
  def self.locked_until(email : String) : Time?
    @@mutex.synchronize do
      entry = @@attempts[email]?
      return nil unless entry

      locked = entry[:locked_until]
      if locked && locked > Time.utc
        locked
      else
        # Lock expired - reset the counter
        @@attempts.delete(email) if locked
        nil
      end
    end
  end

  # Record a failed attempt. Returns the lockout time if the account is now locked.
  def self.record_failure(email : String) : Time?
    max_attempts, lockout_minutes = lockout_settings

    @@mutex.synchronize do
      # Bound memory usage against brute-force with many distinct emails
      @@attempts.clear if @@attempts.size >= MAX_TRACKED_ACCOUNTS

      entry = @@attempts[email]?
      count = entry ? entry[:count] : 0
      count += 1

      locked_until = entry.try(&.[:locked_until])
      if locked_until.nil? && count >= max_attempts
        locked_until = Time.utc + lockout_minutes.minutes
        Log.warn { "Account locked after #{count} failed login attempts: #{email}" }
      end

      @@attempts[email] = {count: count, locked_until: locked_until}
      locked_until
    end
  end

  # Clear the record after a successful login
  def self.record_success(email : String)
    @@mutex.synchronize do
      @@attempts.delete(email)
    end
  end

  private def self.lockout_settings : {Int32, Int32}
    setting = SystemSetting.get("security")
    return {5, 15} unless setting

    parsed = setting.parsed_value
    max_attempts = parsed["max_login_attempts"]?.try(&.as_i?) || 5
    lockout_minutes = parsed["lockout_duration_minutes"]?.try(&.as_i?) || 15
    {max_attempts, lockout_minutes}
  end
end
