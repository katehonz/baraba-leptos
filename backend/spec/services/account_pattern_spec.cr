require "../spec_helper"

describe AccountPattern do
  it "matches exact code and analytical continuation" do
    AccountPattern.matches?("304", "304").should be_true
    AccountPattern.matches?("3041", "304").should be_true
    AccountPattern.matches?("30401", "304").should be_true
    AccountPattern.matches?("305", "304").should be_false
  end

  it "matches a class prefix with star" do
    AccountPattern.matches?("301", "30*").should be_true
    AccountPattern.matches?("304", "30*").should be_true
    AccountPattern.matches?("411", "30*").should be_false
    AccountPattern.matches?("50301", "50*").should be_true
  end

  it "rejects empty pattern or account" do
    AccountPattern.matches?("304", "").should be_false
    AccountPattern.matches?("", "304").should be_false
  end
end
