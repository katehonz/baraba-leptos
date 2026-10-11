require "../../../spec_helper"

describe Api::Reports::Chronological do
  it "shows bank payments that stored only account_id and keeps invoice debit accounts when filtering 401" do
    user = UserFactory.create &.is_super_admin(true)
    company = SaveCompany.create!(name: "Тест ЕООД", eik: "111111111")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Доставчик ЕООД",
      eik: "222222222",
      counterpart_type: "supplier"
    )
    expense = SaveAccount.create!(company_id: company.id, code: "60101", name: "Материали", account_type: "expense")
    vat = SaveAccount.create!(company_id: company.id, code: "4531", name: "ДДС", account_type: "asset")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    bank = SaveAccount.create!(company_id: company.id, code: "50301", name: "Банка", account_type: "asset")

    invoice = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 1, 6),
      document_date: Time.utc(2026, 1, 6),
      document_number: "5000000001",
      description: "Материали",
      status: "posted",
      counterpart_id: counterpart.id,
      lines: [
        {"account_id" => expense.id, "account_code" => "60101", "debit" => 318.64, "credit" => 0.0},
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 318.64},
        {"account_id" => vat.id, "account_code" => "4531", "debit" => 63.73, "credit" => 0.0},
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 63.73},
      ].to_json
    )

    payment = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 1, 26),
      description: "Банково извлечение — SCT Instant издаден превод",
      status: "posted",
      counterpart_id: counterpart.id,
      lines: [
        {"account_id" => supplier.id, "account_code" => "", "debit" => 2700, "credit" => 0.0, "description" => ""},
        {"account_id" => bank.id, "account_code" => "", "debit" => 0.0, "credit" => 2700, "description" => ""},
      ].to_json
    )

    response = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/reports/chronological?date_from=2026-01-01&date_to=2026-01-31&account_code=401&counterpart_id=#{counterpart.id}"
    )
    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    rows = json["data"]["rows"].as_a
    rows.size.should eq(3)

    invoice_rows = rows.select { |r| r["entry_id"].as_i64 == invoice.id }
    invoice_rows.size.should eq(2)
    invoice_rows.each do |row|
      row["credit_account"].as_s.should eq("401")
      row["debit_account"].as_s.should_not eq("")
      row["debit_amount"].as_f.should be > 0
    end
    invoice_rows.map { |r| r["debit_account"].as_s }.sort.should eq(["4531", "60101"])

    pay_rows = rows.select { |r| r["entry_id"].as_i64 == payment.id }
    pay_rows.size.should eq(1)
    pay_rows[0]["debit_account"].as_s.should eq("401")
    pay_rows[0]["credit_account"].as_s.should eq("50301")
    json_amount(pay_rows[0]["debit_amount"]).should eq(2700.0)
    json_amount(pay_rows[0]["credit_amount"]).should eq(2700.0)
    pay_rows[0]["counterpart_name"].as_s.should eq("Доставчик ЕООД")
  end
end

private def json_amount(value : JSON::Any) : Float64
  value.as_f? || value.as_i64?.try(&.to_f) || 0.0
end
