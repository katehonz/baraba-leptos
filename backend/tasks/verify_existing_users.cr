# Run with: lucky verify_existing_users
class VerifyExistingUsers < LuckyTask::Task
  summary "Mark all existing users as email verified"

  def call
    users = UserQuery.new.email_verified_at.is_nil

    count = 0
    users.each do |user|
      SaveUser.update!(user, email_verified_at: Time.utc)
      count += 1
      puts "Verified: #{user.email}"
    end

    puts "Done! Verified #{count} users."
  end
end
