require "../spec_helper"

describe AccountPeriodBalances::Row do
  it "keeps mutations when stored in a Hash (must be a class, not a struct)" do
    rows = {} of Int64 => AccountPeriodBalances::Row
    rows[1_i64] = AccountPeriodBalances::Row.new(1_i64, "601")
    row = rows[1_i64]
    row.turnover_debit += 87.57
    rows[1_i64].turnover_debit.should eq(87.57)
    rows[1_i64].closing_debit.should eq(87.57)
  end
end

describe AccountPeriodBalances do
  it "compares dates by calendar day so the last day is included" do
    AccountPeriodBalances.ymd(Time.utc(2026, 12, 31, 0, 0, 0)).should eq("2026-12-31")
    AccountPeriodBalances.ymd(Time.utc(2026, 12, 31, 23, 59, 59)).should eq("2026-12-31")
  end
end
