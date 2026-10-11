require "../spec_helper"

describe DepreciationEngine do
  it "starts the month after put-into-service and uses annual rate / 12" do
    service = Time.utc(2015, 4, 18)
    params = DepreciationEngine::Params.new(
      cost: 34667.34,
      residual: 0.0,
      rate: 25.0,
      service_date: service
    )
    DepreciationEngine.useful_life_months(25.0).should eq(48)
    DepreciationEngine.monthly_amount(34667.34, 0.0, 25.0).should eq(722.24)

    may = DepreciationEngine.amount_for_month(params, 2015, 5)
    may.should eq(722.24)
    DepreciationEngine.amount_for_month(params, 2015, 4).should eq(0.0)
  end

  it "skips conservation from the month after conserved_from through conserved_to" do
    params = DepreciationEngine::Params.new(
      cost: 3005.00,
      residual: 0.0,
      rate: 30.0,
      service_date: Time.utc(2015, 10, 7),
      conserved_from: Time.utc(2018, 3, 3),
      conserved_to: Time.utc(2018, 9, 15)
    )
    DepreciationEngine.amount_for_month(params, 2018, 3).should be > 0
    (4..9).each do |month|
      row = DepreciationEngine.year_months(params, 2018).find { |r| r.month == month }
      row.should_not be_nil
      row.not_nil!.conserved.should be_true
      row.not_nil!.amount.should eq(0.0)
    end
    DepreciationEngine.amount_for_month(params, 2018, 10).should be > 0
  end

  it "fills twelve months in the first year after put-into-service" do
    params = DepreciationEngine::Params.new(
      cost: 1200.0,
      residual: 0.0,
      rate: 10.0,
      service_date: Time.utc(2025, 12, 1)
    )
    year = DepreciationEngine.year_months(params, 2026)
    year.size.should eq(12)
    year.first.month.should eq(1)
    year.first.amount.should eq(10.0)
    year.map(&.amount).sum.should eq(120.0)
  end
end
