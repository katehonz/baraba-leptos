require "../../../spec_helper"

describe Api::BankAccounts::Create do
  it "creates a bank account from a flat JSON body" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Банка ООД", eik: "121212121")

    body = {
      name:      "Тест Банка Основна",
      bank_name: "Тест Банка",
      iban:      "BG49 TEST 0000 0123 4567 89",
      bic:       "TESTBGSF",
      currency:  "BGN",
    }.to_json

    response = ApiClient.auth(user).exec_raw(
      Api::BankAccounts::Create.with(company.id.to_s),
      body
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["name"].as_s.should eq("Тест Банка Основна")
    json["data"]["iban"].as_s.should eq("BG49TEST00000123456789")
    json["data"]["currency"].as_s.should eq("BGN")
    json["data"]["bank_name"].as_s.should eq("Тест Банка")

    saved = BankAccountQuery.new.company_id(company.id).first
    saved.name.should eq("Тест Банка Основна")
    saved.iban.should eq("BG49TEST00000123456789")
  end

  it "creates a bank account from a nested bank_account payload" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Тест Банка ООД", eik: "131313131")
    gl = SaveAccount.create!(company_id: company.id, code: "503", name: "Разплащателна", account_type: "asset")
    buffer = SaveAccount.create!(company_id: company.id, code: "499", name: "Буфер", account_type: "liability")

    body = {
      bank_account: {
        name:               "Тест Банка EUR",
        iban:               "BG22TEST00000123456790",
        currency:           "EUR",
        gl_account_id:      gl.id,
        buffer_account_id:  buffer.id,
      },
    }.to_json

    response = ApiClient.auth(user).exec_raw(
      Api::BankAccounts::Create.with(company.id.to_s),
      body
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["gl_account_id"].as_i64.should eq(gl.id)
    json["data"]["buffer_account_id"].as_i64.should eq(buffer.id)
  end

  it "returns 422 when name is missing instead of Missing param key" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Без име ООД", eik: "141414141")

    response = ApiClient.auth(user).exec_raw(
      Api::BankAccounts::Create.with(company.id.to_s),
      {iban: "BG92TEST00000123456791", currency: "EUR"}.to_json
    )

    response.status_code.should eq(422)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_false
    json["error"]?.try(&.as_s).should_not eq("Missing param key: 'bank_account'")
  end
end

describe Api::BankAccounts::Update do
  it "updates a bank account from a flat JSON body" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Редакция ООД", eik: "151515151")
    account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Стара сметка",
      iban: "BG65TEST00000123456792",
      currency: "EUR",
      bank_name: "Тест Банка"
    )

    response = ApiClient.auth(user).exec_raw(
      Api::BankAccounts::Update.with(company.id.to_s, account.id.to_s),
      {name: "Нова сметка", iban: "BG38TEST00000123456793", currency: "BGN"}.to_json
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["name"].as_s.should eq("Нова сметка")
    json["data"]["iban"].as_s.should eq("BG38TEST00000123456793")
    json["data"]["currency"].as_s.should eq("BGN")
    json["data"]["bank_name"].as_s.should eq("Тест Банка")
  end
end
