require "../../../spec_helper"

describe Api::Reports::TrialBalanceByCounterpart do
  it "includes bank payments booked only with account_id on the counterpart 401 row" do
    user = UserFactory.create &.is_super_admin(true)
    company = SaveCompany.create!(name: "Обороти ООД", eik: "333333333")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Доставчик ЕООД",
      eik: "444444444",
      counterpart_type: "supplier"
    )
    expense = SaveAccount.create!(company_id: company.id, code: "60101", name: "Материали", account_type: "expense")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    bank = SaveAccount.create!(company_id: company.id, code: "50301", name: "Банка", account_type: "asset")

    SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 1, 6),
      document_date: Time.utc(2026, 1, 6),
      document_number: "5000000001",
      description: "Материали",
      status: "posted",
      counterpart_id: counterpart.id,
      lines: [
        {"account_id" => expense.id, "account_code" => "60101", "debit" => 100.0, "credit" => 0.0},
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 100.0},
      ].to_json
    )

    SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 1, 26),
      description: "Банково извлечение — плащане",
      status: "posted",
      counterpart_id: counterpart.id,
      lines: [
        {"account_id" => supplier.id, "account_code" => "", "debit" => 100.0, "credit" => 0.0},
        {"account_id" => bank.id, "account_code" => "", "debit" => 0.0, "credit" => 100.0},
      ].to_json
    )

    response = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/reports/trial_balance_by_counterpart?date_from=2026-01-01&date_to=2026-01-31&counterpart_id=#{counterpart.id}&account_filter=401"
    )
    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    rows = json["data"]["rows"].as_a
    rows.size.should eq(1)
    row = rows[0]
    row["account_code"].as_s.should eq("401")
    json_amount(row["turnover_debit"]).should eq(100.0)
    json_amount(row["turnover_credit"]).should eq(100.0)
  end
end

private def json_amount(value : JSON::Any) : Float64
  value.as_f? || value.as_i64?.try(&.to_f) || 0.0
end
