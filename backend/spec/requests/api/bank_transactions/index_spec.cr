require "../../../spec_helper"

describe Api::BankTransactions::Index do
  it "filters by a small date range and paginates" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Дати ООД", eik: "161616161")
    account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Тест Банка",
      iban: "BG54TEST00000123456796",
      currency: "BGN"
    )

    SaveBankTransaction.create!(
      bank_account_id: account.id,
      date: Time.utc(2026, 3, 1),
      amount: -10.0,
      currency: "BGN",
      description: "Март 1"
    )
    in_range = SaveBankTransaction.create!(
      bank_account_id: account.id,
      date: Time.utc(2026, 3, 10),
      amount: -20.0,
      currency: "BGN",
      description: "Март 10"
    )
    SaveBankTransaction.create!(
      bank_account_id: account.id,
      date: Time.utc(2026, 3, 15),
      amount: -30.0,
      currency: "BGN",
      description: "Март 15"
    )

    response = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/bank_transactions?date_from=2026-03-10&date_to=2026-03-12&per_page=25"
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    rows = json["data"].as_a
    rows.size.should eq(1)
    rows[0]["id"].as_i64.should eq(in_range.id)
    rows[0]["description"].as_s.should eq("Март 10")
    json["pagination"]["total"].as_i64.should eq(1)
    json["pagination"]["page"].as_i64.should eq(1)
  end

  it "returns pagination metadata across pages" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Страници ООД", eik: "171717171")
    account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Тест Банка",
      iban: "BG79TEST00000134567890",
      currency: "BGN"
    )

    3.times do |i|
      SaveBankTransaction.create!(
        bank_account_id: account.id,
        date: Time.utc(2026, 4, i + 1),
        amount: -(i + 1).to_f,
        currency: "BGN",
        description: "Ред #{i + 1}"
      )
    end

    page1 = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/bank_transactions?per_page=2&page=1"
    )
    json1 = JSON.parse(page1.body)
    json1["data"].as_a.size.should eq(2)
    json1["pagination"]["total"].as_i64.should eq(3)
    json1["pagination"]["total_pages"].as_i64.should eq(2)
    json1["pagination"]["per_page"].as_i64.should eq(2)

    page2 = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/bank_transactions?per_page=2&page=2"
    )
    json2 = JSON.parse(page2.body)
    json2["data"].as_a.size.should eq(1)
    json2["pagination"]["page"].as_i64.should eq(2)
  end

  it "filters unallocated transactions that still use the buffer account" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Буфер ООД", eik: "181818181")
    gl = SaveAccount.create!(company_id: company.id, code: "503", name: "Банка", account_type: "asset")
    buffer = SaveAccount.create!(company_id: company.id, code: "499", name: "Буфер", account_type: "liability")
    other = SaveAccount.create!(company_id: company.id, code: "602", name: "Разход", account_type: "expense")
    account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Тест Банка",
      iban: "BG52TEST00000134567891",
      currency: "BGN",
      gl_account_id: gl.id,
      buffer_account_id: buffer.id
    )

    pending_je = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 5, 1),
      description: "Буфер",
      status: "draft",
      lines: [
        {"account_id" => buffer.id, "debit" => 50.0, "credit" => 0.0},
        {"account_id" => gl.id, "debit" => 0.0, "credit" => 50.0},
      ].to_json
    )
    done_je = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 5, 2),
      description: "Разход",
      status: "draft",
      lines: [
        {"account_id" => other.id, "debit" => 50.0, "credit" => 0.0},
        {"account_id" => gl.id, "debit" => 0.0, "credit" => 50.0},
      ].to_json
    )

    pending_tx = SaveBankTransaction.create!(
      bank_account_id: account.id,
      date: Time.utc(2026, 5, 1),
      amount: -50.0,
      currency: "BGN",
      description: "За обработка",
      journal_entry_id: pending_je.id
    )
    SaveBankTransaction.create!(
      bank_account_id: account.id,
      date: Time.utc(2026, 5, 2),
      amount: -50.0,
      currency: "BGN",
      description: "Обработено",
      journal_entry_id: done_je.id
    )

    response = ApiClient.auth(user).get(
      "/api/companies/#{company.id}/bank_transactions?status=unallocated"
    )
    json = JSON.parse(response.body)
    rows = json["data"].as_a
    rows.size.should eq(1)
    rows[0]["id"].as_i64.should eq(pending_tx.id)
    rows[0]["is_allocated"].as_bool.should be_false
  end
end
