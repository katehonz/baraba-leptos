require "../spec_helper"

describe VatAmounts do
  it "computes 20% tax base as gross / 1.20" do
    VatAmounts.tax_base_from_gross(120.0).should be_close(100.0, 0.001)
    VatAmounts.vat_from_base(100.0).should be_close(20.0, 0.001)
    VatAmounts.tax_base_from_gross(100.0).should be_close(83.33, 0.001)
  end

  it "accepts matching 20% amounts" do
    check = VatAmounts.check_amounts(100.0, 20.0, 120.0)
    check[:ok].should be_true
    check[:messages].should be_empty
  end

  it "flags tax base that was incorrectly taken as net minus VAT" do
    check = VatAmounts.check_amounts(80.0, 20.0, 120.0)
    check[:ok].should be_false
    check[:expected_base].should be_close(100.0, 0.001)
    check[:expected_vat].should be_close(20.0, 0.001)
    check[:messages].join.should contain("1,20")
  end

  it "resolves AI-scan journal rows that stored NET as total_amount" do
    company = SaveCompany.create!(name: "ДДС AI ООД", eik: "811111111")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Фактура 0001",
      document_number: "0001",
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

    resolved = VatAmounts.from_journal_entry(entry)
    resolved[:base].should be_close(100.0, 0.001)
    resolved[:vat].should be_close(20.0, 0.001)
    resolved[:gross].should be_close(120.0, 0.001)
  end

  it "resolves AI-scan rows when the VAT debit line is missing" do
    company = SaveCompany.create!(name: "ДДС Нет ООД", eik: "811111112")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Фактура без 453",
      document_number: "0002",
      status: "draft",
      vat_period: "2026-03",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      total_amount: 100.0,
      total_vat_amount: 20.0,
      lines: %([
        {"account_code":"602","debit":100.0,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":120.0}
      ])
    )

    resolved = VatAmounts.from_journal_entry(entry)
    resolved[:base].should be_close(100.0, 0.001)
    resolved[:vat].should be_close(20.0, 0.001)
    resolved[:gross].should be_close(120.0, 0.001)
  end

  it "resolves manual journal rows that stored GROSS as total_amount" do
    company = SaveCompany.create!(name: "ДДС Бруто ООД", eik: "811111113")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Ръчна статия",
      document_number: "0003",
      status: "draft",
      vat_period: "2026-03",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      total_amount: 120.0,
      total_vat_amount: 20.0,
      lines: %([
        {"account_code":"602","debit":100.0,"credit":0.0},
        {"account_code":"4531","debit":20.0,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":120.0}
      ])
    )

    resolved = VatAmounts.from_journal_entry(entry)
    resolved[:base].should be_close(100.0, 0.001)
    resolved[:vat].should be_close(20.0, 0.001)
    resolved[:gross].should be_close(120.0, 0.001)
  end

  it "resolves 9% VAT from gross" do
    company = SaveCompany.create!(name: "ДДС 9 ООД", eik: "811111114")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Хотел",
      document_number: "0004",
      status: "draft",
      vat_period: "2026-03",
      vat_purchase_operation: "01|PURCHASE_CREDIT",
      total_amount: 109.0,
      total_vat_amount: 9.0,
      lines: %([
        {"account_code":"602","debit":100.0,"credit":0.0},
        {"account_code":"4531","debit":9.0,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":109.0}
      ])
    )

    resolved = VatAmounts.from_journal_entry(entry)
    resolved[:base].should be_close(100.0, 0.001)
    resolved[:vat].should be_close(9.0, 0.001)
    resolved[:gross].should be_close(109.0, 0.001)
  end
end
