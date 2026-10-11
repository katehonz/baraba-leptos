require "../../../spec_helper"

private def auth_user : User
  UserFactory.create &.is_super_admin(true)
end

private def make_company(name : String, eik : String, currency : String = "EUR")
  SaveCompany.create!(name: name, eik: eik, currency: currency)
end

private def make_counterpart(company : Company, name : String = "Клиент ООД")
  SaveCounterpart.create!(
    company_id: company.id,
    name: name,
    counterpart_type: "customer"
  )
end

private def post_invoice(user : User, company : Company, body : String)
  ApiClient.auth(user).exec_raw(
    Api::Invoices::Create.with(company.id.to_s),
    body
  )
end

describe Api::Invoices::Create do
  it "creates a sales draft without invoice_number and uses company currency" do
    user = auth_user
    company = make_company("Фактури ООД", "201111111", "EUR")
    counterpart = make_counterpart(company)

    response = post_invoice(user, company, {
      counterpart_id: counterpart.id,
      issue_date:     "2026-09-01",
      subtotal:       100.0,
      vat_amount:     20.0,
      total_amount:   120.0,
    }.to_json)

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["invoice_number"].raw.should be_nil
    json["data"]["direction"].as_s.should eq("sale")
    json["data"]["document_type"].as_s.should eq("INVOICE")
    json["data"]["status"].as_s.should eq("draft")
    json["data"]["currency"].as_s.should eq("EUR")
    json["data"]["issue_date"].as_s.should eq("2026-09-01")

    saved = InvoiceQuery.new.company_id(company.id).first
    saved.invoice_number.should be_nil
    saved.direction.should eq("sale")
    saved.currency.should eq("EUR")
  end

  it "allows two sales drafts with null invoice_number" do
    user = auth_user
    company = make_company("Две чернови ООД", "201111112")
    counterpart = make_counterpart(company)

    2.times do
      response = post_invoice(user, company, {
        counterpart_id: counterpart.id,
        issue_date:     "2026-09-02",
        subtotal:       10.0,
        vat_amount:     2.0,
        total_amount:   12.0,
      }.to_json)
      response.status_code.should eq(200)
    end

    InvoiceQuery.new.company_id(company.id).select_count.should eq(2)
  end

  it "rejects a second sale with the same invoice_number" do
    user = auth_user
    company = make_company("Уникален номер ООД", "201111113")
    counterpart = make_counterpart(company)

    body = {
      counterpart_id:  counterpart.id,
      invoice_number:  "0000000001",
      issue_date:      "2026-09-03",
      subtotal:        10.0,
      vat_amount:      2.0,
      total_amount:    12.0,
    }.to_json

    post_invoice(user, company, body).status_code.should eq(200)
    response = post_invoice(user, company, body)
    response.status_code.should eq(422)
  end

  it "allows a purchase to reuse a sales invoice_number" do
    user = auth_user
    company = make_company("Покупка номер ООД", "201111114")
    counterpart = make_counterpart(company)

    post_invoice(user, company, {
      counterpart_id: counterpart.id,
      invoice_number: "0000000007",
      direction:      "sale",
      issue_date:     "2026-09-04",
      subtotal:       10.0,
      vat_amount:     2.0,
      total_amount:   12.0,
    }.to_json).status_code.should eq(200)

    response = post_invoice(user, company, {
      counterpart_id: counterpart.id,
      invoice_number: "0000000007",
      direction:      "purchase",
      issue_date:     "2026-09-04",
      subtotal:       10.0,
      vat_amount:     2.0,
      total_amount:   12.0,
    }.to_json)

    response.status_code.should eq(200)
    JSON.parse(response.body)["data"]["direction"].as_s.should eq("purchase")
  end

  it "returns 403 without invoice:create" do
    user = UserFactory.create
    company = make_company("Без права ООД", "201111115")
    counterpart = make_counterpart(company)

    response = post_invoice(user, company, {
      counterpart_id: counterpart.id,
      issue_date:     "2026-09-05",
      subtotal:       1.0,
      vat_amount:     0.2,
      total_amount:   1.2,
    }.to_json)

    response.status_code.should eq(403)
  end

  it "accepts a nested invoice payload" do
    user = auth_user
    company = make_company("Вложен JSON ООД", "201111116")
    counterpart = make_counterpart(company)

    response = post_invoice(user, company, {
      invoice: {
        counterpart_id: counterpart.id,
        issue_date:     "2026-09-06",
        subtotal:       50.0,
        vat_amount:     10.0,
        total_amount:   60.0,
        document_type:  "PROFORMA",
      },
    }.to_json)

    response.status_code.should eq(200)
    JSON.parse(response.body)["data"]["document_type"].as_s.should eq("PROFORMA")
  end

  it "rejects a 0% line without a VAT exemption reason" do
    user = auth_user
    company = make_company("Нулева ставка ООД", "201111121")
    counterpart = make_counterpart(company)

    response = post_invoice(user, company, {
      counterpart_id: counterpart.id,
      issue_date:     "2026-09-12",
      lines:          [{
        description: "Услуга",
        quantity:    1,
        unit_price:  100,
        vat_rate:    0,
      }],
    }.to_json)

    response.status_code.should eq(422)
    JSON.parse(response.body)["error"].as_s.should contain("основание")
  end

  it "saves a 0% line with an exemption reason" do
    user = auth_user
    company = make_company("Чл. 113 ООД", "201111122")
    counterpart = make_counterpart(company)
    reason = SaveVatExemptionReason.create!(
      reason_code: "VATEX-EU-SM-TEST",
      reason_name: "Малко предприятие",
      legal_basis: "чл. 113 от ЗДДС",
      ubl_category_code: "E",
      ubl_exemption_code: "vatex-eu-e",
      is_active: true,
      sort_order: 40
    )

    response = post_invoice(user, company, {
      counterpart_id: counterpart.id,
      issue_date:     "2026-09-12",
      currency:       "EUR",
      lines:          [{
        description:              "Счетоводна услуга",
        quantity:                 1,
        unit_price:               200,
        vat_rate:                 0,
        vat_exemption_reason_id:  reason.id,
        unit_code:                "HUR",
      }],
    }.to_json)

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["vat_amount"].as_f.should eq(0.0)
    json["data"]["has_inventory"].as_bool.should be_false
    json["data"]["vat_exemption_reason"].as_s.should eq("Малко предприятие")
  end
end

describe Api::Invoices::Index do
  it "includes direction so purchases are distinguishable from sales" do
    user = auth_user
    company = make_company("Списък ООД", "201111117")
    counterpart = make_counterpart(company)

    SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 7),
      subtotal: 10.0,
      vat_amount: 2.0,
      total_amount: 12.0,
      direction: "sale",
      status: "posted",
      invoice_number: "0000000010"
    )
    SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 7),
      subtotal: 10.0,
      vat_amount: 2.0,
      total_amount: 12.0,
      direction: "purchase",
      status: "posted",
      invoice_number: "0000000010"
    )

    response = ApiClient.auth(user).exec_raw(
      Api::Invoices::Index.with(company.id.to_s),
      ""
    )
    response.status_code.should eq(200)
    data = JSON.parse(response.body)["data"].as_a
    data.size.should eq(2)
    directions = data.map { |row| row["direction"].as_s }.sort
    directions.should eq(["purchase", "sale"])
  end
end

describe Api::Invoices::Delete do
  it "deletes a draft and rejects a posted invoice" do
    user = auth_user
    company = make_company("Изтриване ООД", "201111118")
    counterpart = make_counterpart(company)

    draft = SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 8),
      subtotal: 10.0,
      vat_amount: 2.0,
      total_amount: 12.0,
      status: "draft"
    )
    posted = SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 8),
      subtotal: 10.0,
      vat_amount: 2.0,
      total_amount: 12.0,
      status: "posted",
      invoice_number: "0000000020"
    )

    del_draft = ApiClient.auth(user).exec_raw(
      Api::Invoices::Delete.with(company.id.to_s, draft.id.to_s),
      ""
    )
    del_draft.status_code.should eq(200)
    InvoiceQuery.new.id(draft.id).first?.should be_nil

    del_posted = ApiClient.auth(user).exec_raw(
      Api::Invoices::Delete.with(company.id.to_s, posted.id.to_s),
      ""
    )
    del_posted.status_code.should eq(422)
    InvoiceQuery.new.id(posted.id).first?.should_not be_nil
  end
end

describe Api::Invoices::Show do
  it "returns parties, lines and amount in words" do
    user = auth_user
    company = make_company("Печат ООД", "201111120", "BGN")
    counterpart = make_counterpart(company, "Клиент АД")

    invoice = SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 11),
      subtotal: 100.0,
      vat_amount: 20.0,
      total_amount: 120.0,
      direction: "sale",
      status: "issued",
      invoice_number: "0000000040",
      currency: "BGN"
    )
    SaveInvoiceLine.create!(
      invoice_id: invoice.id,
      description: "Услуга",
      quantity: 1.0,
      unit_price: 100.0,
      vat_rate: 20.0,
      unit_code: "C62",
      line_number: 1
    )

    response = ApiClient.auth(user).exec_raw(
      Api::Invoices::Show.with(company.id.to_s, invoice.id.to_s),
      ""
    )
    response.status_code.should eq(200)
    data = JSON.parse(response.body)["data"]
    data["company_name"].as_s.should eq("Печат ООД")
    data["counterpart_name"].as_s.should eq("Клиент АД")
    data["amount_in_words"].as_s.should eq("Сто и двадесет лева")
    data["lines"].as_a.size.should eq(1)
    data["lines"].as_a.first["description"].as_s.should eq("Услуга")
  end
end

describe "SAF-T sales invoices direction filter" do
  it "excludes posted purchases from the sales query" do
    company = make_company("САФТ ООД", "201111119")
    counterpart = make_counterpart(company)

    SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 10),
      subtotal: 100.0,
      vat_amount: 20.0,
      total_amount: 120.0,
      direction: "sale",
      status: "posted",
      invoice_number: "0000000030",
      document_type: "INVOICE"
    )
    SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: counterpart.id,
      issue_date: Time.utc(2026, 9, 10),
      subtotal: 80.0,
      vat_amount: 16.0,
      total_amount: 96.0,
      direction: "purchase",
      status: "posted",
      invoice_number: "0000000030",
      document_type: "INVOICE"
    )

    sales = InvoiceQuery.new.company_id(company.id).status("posted").direction("sale")
    sales.select_count.should eq(1)
    sales.first.invoice_number.should eq("0000000030")
    sales.first.direction.should eq("sale")
  end
end
