require "../../../spec_helper"

describe Api::VatReturns::ImportFromJournal do
  it "imports AI-scan NET total_amount as the tax base, not net minus VAT" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "Импорт ДДС ООД", eik: "822222221", vat_number: "BG822222221")
    vat_return = SaveVatReturn.create!(
      company_id: company.id,
      period_year: 2026,
      period_month: 3,
      status: "draft"
    )
    SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 10),
      document_date: Time.utc(2026, 3, 10),
      description: "Фактура AI-1",
      document_number: "AI-1",
      status: "draft",
      vat_period: "2026-03",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      total_amount: 100.0,
      total_vat_amount: 20.0,
      lines: %([
        {"account_code":"602","debit":100.0,"credit":0.0},
        {"account_code":"4531","debit":20.0,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":120.0}
      ])
    )

    response = ApiClient.auth(user).exec_raw(
      Api::VatReturns::ImportFromJournal.with(company.id.to_s, vat_return.id.to_s),
      "{}"
    )

    response.status_code.should eq(200)
    json = JSON.parse(response.body)
    json["success"].as_bool.should be_true
    (json["data"]["purchase_count"].as_i64? || json["data"]["purchase_count"].as_i.to_i64).should eq(1)

    imported = VatJournalEntryQuery.new.vat_return_id(vat_return.id).entry_type("purchase").first
    imported.base_full_credit.should be_close(100.0, 0.001)
    imported.vat_full_credit.should be_close(20.0, 0.001)

    calculated = VatReturnQuery.new.find(vat_return.id)
    calculated.cell_01_21.should be_close(100.0, 0.001)
    calculated.cell_01_22.should be_close(20.0, 0.001)
    calculated.cell_01_30.should be_close(0.0, 0.001)

    result = JSON.parse(calculated.result_json.not_nil!)
    json_f64(result["purchase_base_20"]).should be_close(100.0, 0.001)
    json_f64(result["purchase_vat_20"]).should be_close(20.0, 0.001)
  end

  it "stores invoice GROSS on the journal entry created from a scanned invoice" do
    company = SaveCompany.create!(name: "Скан ДДС ООД", eik: "822222222")
    expense = SaveAccount.create!(company_id: company.id, code: "602", name: "Разходи", account_type: "expense")
    vat_acc = SaveAccount.create!(company_id: company.id, code: "4531", name: "ДДС", account_type: "asset")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    SaveCompany.update!(company,
      expenses_account_id: expense.id,
      vat_receivable_account_id: vat_acc.id,
      suppliers_account_id: supplier.id
    )
    company = CompanyQuery.new.find(company.id)

    invoice = SaveScannedInvoice.create!(
      company_id: company.id,
      direction: "purchase",
      status: "pending",
      vies_status: "pending",
      vendor_name: "Доставчик",
      invoice_number: "SC-9",
      invoice_date: Time.utc(2026, 3, 12),
      vat_period: "2026-03",
      subtotal: 100.0,
      total_tax: 20.0,
      invoice_total: 120.0
    )

    booked = ScannedInvoiceAccounting.book_invoice!(invoice, company)
    entry = JournalEntryQuery.new.find(booked.journal_entry_id.not_nil!)
    entry.total_amount.not_nil!.should be_close(120.0, 0.001)
    entry.total_vat_amount.not_nil!.should be_close(20.0, 0.001)

    resolved = VatAmounts.from_journal_entry(entry)
    resolved[:base].should be_close(100.0, 0.001)
    resolved[:vat].should be_close(20.0, 0.001)
  end
end

private def json_f64(value : JSON::Any) : Float64
  value.as_f? || value.as_i64?.try(&.to_f) || value.as_i?.try(&.to_f) || 0.0
end
