require "../spec_helper"

describe AccountCodes do
  it "orders by account class then code, not as integers" do
    codes = ["703", "7029", "45901", "50301", "401", "4011"]
    sorted = codes.sort { |a, b| AccountCodes.compare(a, b) }
    sorted.should eq(["401", "4011", "45901", "50301", "7029", "703"])
  end

  it "keeps a synthetic account before its analytical accounts" do
    codes = ["7029", "702", "703", "70"]
    sorted = codes.sort { |a, b| AccountCodes.compare(a, b) }
    sorted.should eq(["70", "702", "7029", "703"])
  end
end
