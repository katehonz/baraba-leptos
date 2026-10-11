class SignInUser
  getter email : String
  getter password : String
  getter errors : Hash(String, Array(String))

  def initialize(@email : String, @password : String)
    @errors = {} of String => Array(String)
  end

  def self.run(params : Lucky::Params, &)
    # Try nested :user first, then root params
    user_hash = params.nested?(:user)
    if user_hash
      email = user_hash["email"]? || ""
      password = user_hash["password"]? || ""
    else
      email = params.get?(:email).to_s
      password = params.get?(:password).to_s
    end

    operation = new(email, password)
    user = operation.submit
    yield operation, user
  end

  def submit : User?
    if email.blank?
      @errors["email"] = ["is required"]
    end
    if password.blank?
      @errors["password"] = ["is required"]
    end

    return nil if @errors.any?

    user = UserQuery.new.email(email).first?

    if user && Authentic.correct_password?(user, password)
      user
    else
      @errors["email"] = ["or password is incorrect"]
      nil
    end
  end
end
