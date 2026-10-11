require "../../../spec_helper"

describe Api::ScannedInvoices::Update do
  it "saves corrected tax base, VAT and total from a flat JSON body with numbers" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Тест ООД", eik: "123456789")
    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending",
      vendor_name: "Доставчик",
      invoice_number: "0001",
      subtotal: 10.0,
      total_tax: 2.0,
      invoice_total: 12.0
    )

    body = {
      vendor_name:   "Коригиран доставчик",
      subtotal:      1234.56,
      total_tax:     246.91,
      invoice_total: 1481.47,
      invoice_date:  "2026-03-15",
    }.to_json

    response = ApiClient.auth(user).exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      body
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    json["data"]["vendor_name"].as_s.should eq("Коригиран доставчик")
    json_f64(json["data"]["subtotal"]).should be_close(1234.56, 0.001)
    json_f64(json["data"]["total_tax"]).should be_close(246.91, 0.001)
    json_f64(json["data"]["invoice_total"]).should be_close(1481.47, 0.001)

    reloaded = ScannedInvoiceQuery.new.find(invoice.id)
    reloaded.vendor_name.should eq("Коригиран доставчик")
    reloaded.subtotal.not_nil!.should be_close(1234.56, 0.001)
    reloaded.total_tax.not_nil!.should be_close(246.91, 0.001)
    reloaded.invoice_total.not_nil!.should be_close(1481.47, 0.001)
    reloaded.invoice_date.not_nil!.to_s("%Y-%m-%d").should eq("2026-03-15")
  end

  it "accepts nested scanned_invoice payload and integer amounts" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Тест 2 ООД", eik: "987654321")
    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending",
      subtotal: 1.0,
      total_tax: 0.0,
      invoice_total: 1.0
    )

    body = {
      scanned_invoice: {
        subtotal:      100,
        total_tax:     20,
        invoice_total: 120,
      },
    }.to_json

    response = ApiClient.auth(user).exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      body
    )

    response.status_code.should eq(200)
    reloaded = ScannedInvoiceQuery.new.find(invoice.id)
    reloaded.subtotal.not_nil!.should be_close(100.0, 0.001)
    reloaded.total_tax.not_nil!.should be_close(20.0, 0.001)
    reloaded.invoice_total.not_nil!.should be_close(120.0, 0.001)
  end

  it "links a counterpart from this company and stores the VIES result" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Тест контрагент ООД", eik: "204567890")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Доставчик ЕООД",
      eik: "131272801",
      vat_number: "BG131272801",
      counterpart_type: "supplier"
    )
    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending",
      vendor_vat_number: "BG131272801"
    )

    body = {
      counterpart_id:           counterpart.id,
      vies_status:              "valid",
      vies_validation_message:  "Намерен в базата данни",
      vies_company_name:        counterpart.name,
      vies_company_address:     "София",
    }.to_json

    response = ApiClient.auth(user).exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      body
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["data"]["counterpart_id"].as_i64.should eq(counterpart.id)
    json["data"]["vies_status"].as_s.should eq("valid")
    json["data"]["vies_validation_message"].as_s.should eq("Намерен в базата данни")

    reloaded = ScannedInvoiceQuery.new.find(invoice.id)
    reloaded.counterpart_id.should eq(counterpart.id)
    reloaded.vies_status.should eq("valid")
    reloaded.vies_company_name.should eq(counterpart.name)
    reloaded.vies_validated_at.should_not be_nil
  end

  it "clears the counterpart and rejects a counterpart from another company" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Тест чистене ООД", eik: "304567890")
    other = SaveCompany.create!(name: "Друга фирма ООД", eik: "404567890")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Наш доставчик",
      eik: "175123456",
      counterpart_type: "supplier"
    )
    foreign = SaveCounterpart.create!(
      company_id: other.id,
      name: "Чужд доставчик",
      eik: "175123457",
      counterpart_type: "supplier"
    )
    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending",
      counterpart_id: counterpart.id
    )

    cleared = ApiClient.auth(user).exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      {counterpart_id: nil}.to_json
    )
    cleared.status_code.should eq(200)
    ScannedInvoiceQuery.new.find(invoice.id).counterpart_id.should be_nil

    rejected = ApiClient.auth(user).exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      {counterpart_id: foreign.id}.to_json
    )
    rejected.status_code.should eq(422)
    ScannedInvoiceQuery.new.find(invoice.id).counterpart_id.should be_nil
  end

  it "rejects unauthenticated updates" do
    company = SaveCompany.create!(name: "Тест 3 ООД", eik: "111222333")
    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending"
    )

    response = ApiClient.new.exec_raw(
      Api::ScannedInvoices::Update.with(company.id.to_s, invoice.id.to_s),
      {subtotal: 1}.to_json
    )

    response.status_code.should eq(401)
  end
end

private def json_f64(value : JSON::Any) : Float64
  value.as_f? || value.as_i64?.try(&.to_f) || value.as_i?.try(&.to_f) || 0.0
end
