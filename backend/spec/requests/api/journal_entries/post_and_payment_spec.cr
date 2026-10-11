require "../../../spec_helper"

describe "Journal entry post and payment" do
  it "posts a draft that stores lines only as JSON (no journal_lines rows)" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Пост ООД", eik: "101010101")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Гориво",
      document_number: "K-12",
      status: "draft",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      lines: %([
        {"account_code":"602","debit":50.0,"credit":0.0},
        {"account_code":"4531","debit":10.0,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":60.0}
      ])
    )

    response = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::Post.with(company.id.to_s, entry.id.to_s),
      "{}"
    )

    response.status_code.should eq(200)
    JSON.parse(response.body)["success"].as_bool.should be_true
    JournalEntryQuery.new.find(entry.id).status.should eq("posted")
  end

  it "keeps document_number when the bank editor omits it" do
    user = UserFactory.create &.is_super_admin(true)
    company = SaveCompany.create!(name: "Док Номер ООД", eik: "101010102")
    acc401 = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    acc503 = SaveAccount.create!(company_id: company.id, code: "503", name: "Банка", account_type: "asset")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 1, 26),
      document_date: Time.utc(2026, 1, 26),
      document_number: "1257245",
      description: "Банково извлечение",
      status: "draft",
      lines: [
        {"account_id" => acc401.id, "debit" => 100.0, "credit" => 0.0},
        {"account_id" => acc503.id, "debit" => 0.0, "credit" => 100.0},
      ].to_json
    )

    response = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::Update.with(company.id.to_s, entry.id.to_s),
      {
        journal_entry: {
          description: "Банково извлечение — плащане",
          counterpart_id: nil,
          lines: [
            {"account_id" => acc401.id, "debit" => 100.0, "credit" => 0.0},
            {"account_id" => acc503.id, "debit" => 0.0, "credit" => 100.0},
          ],
          total_amount: 100.0,
          status: "draft",
        },
      }.to_json
    )
    response.status_code.should eq(200)
    saved = JournalEntryQuery.new.find(entry.id)
    saved.document_number.should eq("1257245")
    saved.description.should eq("Банково извлечение — плащане")
    JournalLines.account_code(JournalLines.parse(saved.lines)[0]).should eq("401")
  end

  it "creates a cash payment JE for a manual purchase (Dt supplier / Ct cash)" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Каса ООД", eik: "202020202")
    cash = SaveAccount.create!(company_id: company.id, code: "501", name: "Каса", account_type: "asset")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    expense = SaveAccount.create!(company_id: company.id, code: "602", name: "Горива", account_type: "expense")
    SaveCompany.update!(company, cash_account_id: cash.id, suppliers_account_id: supplier.id, expenses_account_id: expense.id)

    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Бензиностанция",
      document_number: "K-88",
      status: "draft",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      lines: [
        {"account_id" => expense.id, "account_code" => "602", "debit" => 83.33, "credit" => 0.0},
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 83.33},
      ].to_json
    )

    response = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::CreatePayment.with(company.id.to_s, entry.id.to_s),
      {method: "cash"}.to_json
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    pay_id = json["data"]["payment_journal_entry_id"].as_i64
    pay = JournalEntryQuery.new.find(pay_id)
    pay.reference.should eq("PAY-#{entry.id}")
    pay.payment_method_code.should eq("10")
    pay.status.should eq("draft")
    lines = JournalLines.parse(pay.lines)
    lines.size.should eq(2)
    JournalLines.debit(lines[0]).should be_close(83.33, 0.001)
    JournalLines.credit(lines[1]).should be_close(83.33, 0.001)
  end

  it "creates a card payment JE for a sale (Dt card buffer / Ct customer)" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "ПОС ООД", eik: "303030303")
    card = SaveAccount.create!(company_id: company.id, code: "498", name: "Карта продажби", account_type: "asset")
    customer = SaveAccount.create!(company_id: company.id, code: "411", name: "Клиенти", account_type: "asset")
    revenue = SaveAccount.create!(company_id: company.id, code: "701", name: "Приходи", account_type: "revenue")
    SaveCompany.update!(company, card_sales_account_id: card.id, customers_account_id: customer.id, revenues_account_id: revenue.id)

    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Продажба POS",
      document_number: "F-9",
      status: "draft",
      vat_sales_operation: "01|DOMESTIC_SALE",
      lines: [
        {"account_id" => customer.id, "account_code" => "411", "debit" => 120.0, "credit" => 0.0},
        {"account_id" => revenue.id, "account_code" => "701", "debit" => 0.0, "credit" => 120.0},
      ].to_json
    )

    response = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::CreatePayment.with(company.id.to_s, entry.id.to_s),
      {method: "card"}.to_json
    )

    response.status_code.should eq(200)
    pay_id = JSON.parse(response.body)["data"]["payment_journal_entry_id"].as_i64
    pay = JournalEntryQuery.new.find(pay_id)
    pay.payment_method_code.should eq("48")
    pay.reference.should eq("PAY-#{entry.id}")
  end

  it "rejects a second payment for the same entry" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Дубль ООД", eik: "404040404")
    cash = SaveAccount.create!(company_id: company.id, code: "501", name: "Каса", account_type: "asset")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    expense = SaveAccount.create!(company_id: company.id, code: "602", name: "Горива", account_type: "expense")
    SaveCompany.update!(company, cash_account_id: cash.id, suppliers_account_id: supplier.id)

    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Касова бележка",
      status: "draft",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      lines: [
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 20.0},
        {"account_id" => expense.id, "account_code" => "602", "debit" => 20.0, "credit" => 0.0},
      ].to_json
    )

    first = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::CreatePayment.with(company.id.to_s, entry.id.to_s),
      {method: "cash"}.to_json
    )
    first.status_code.should eq(200)

    second = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::CreatePayment.with(company.id.to_s, entry.id.to_s),
      {method: "cash"}.to_json
    )
    second.status_code.should eq(400)
    JSON.parse(second.body)["error"].as_s.should contain("Вече има статия за плащане")
  end

  it "returns a clear error when cash account is not configured" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Без каса ООД", eik: "505050505")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    SaveCompany.update!(company, suppliers_account_id: supplier.id)

    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Гориво",
      status: "draft",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      lines: [
        {"account_id" => supplier.id, "account_code" => "401", "debit" => 0.0, "credit" => 10.0},
        {"account_id" => supplier.id, "account_code" => "602", "debit" => 10.0, "credit" => 0.0},
      ].to_json
    )

    response = ApiClient.auth(user).exec_raw(
      Api::JournalEntries::CreatePayment.with(company.id.to_s, entry.id.to_s),
      {method: "cash"}.to_json
    )
    response.status_code.should eq(400)
    JSON.parse(response.body)["error"].as_s.should contain("Каса")
  end
end
