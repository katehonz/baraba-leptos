require "../spec_helper"

describe BankTransactionAllocator do
  it "books a withdrawal with account_code, document date and counterpart on 401" do
    company = SaveCompany.create!(name: "Банка Алок ООД", eik: "191919191")
    gl = SaveAccount.create!(company_id: company.id, code: "503", name: "Банка", account_type: "asset")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Доставчик ЕООД",
      counterpart_type: "supplier"
    )
    bank_account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Тест Банка",
      iban: "BG11TEST00000123456794",
      currency: "BGN",
      gl_account_id: gl.id
    )
    tx = SaveBankTransaction.create!(
      bank_account_id: bank_account.id,
      date: Time.utc(2026, 1, 26),
      amount: -2700.0,
      currency: "BGN",
      description: "SCT Instant издаден превод",
      reference: "1257245",
      contra_name: "Доставчик ЕООД"
    )

    entry = BankTransactionAllocator.book!(company.id, tx, supplier.id, counterpart.id)
    entry.document_date.should_not be_nil
    entry.document_date.not_nil!.to_s("%Y-%m-%d").should eq("2026-01-26")
    entry.document_number.should eq("1257245")
    entry.counterpart_id.should eq(counterpart.id)

    lines = JournalLines.parse(entry.lines)
    lines.size.should eq(2)
    JournalLines.account_code(lines[0]).should eq("401")
    JournalLines.debit(lines[0]).should eq(2700.0)
    JournalLines.counterpart_id(lines[0]).should eq(counterpart.id)
    JournalLines.account_code(lines[1]).should eq("503")
    JournalLines.credit(lines[1]).should eq(2700.0)
  end

  it "fills account_code when reallocating the buffer line" do
    company = SaveCompany.create!(name: "Буфер Алок ООД", eik: "191919192")
    gl = SaveAccount.create!(company_id: company.id, code: "503", name: "Банка", account_type: "asset")
    buffer = SaveAccount.create!(company_id: company.id, code: "499", name: "Буфер", account_type: "liability")
    supplier = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    counterpart = SaveCounterpart.create!(
      company_id: company.id,
      name: "Доставчик ЕООД",
      counterpart_type: "supplier"
    )
    bank_account = SaveBankAccount.create!(
      company_id: company.id,
      name: "Тест Банка",
      iban: "BG81TEST00000123456795",
      currency: "BGN",
      gl_account_id: gl.id,
      buffer_account_id: buffer.id
    )
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 9),
      description: "Банково извлечение",
      status: "draft",
      lines: [
        {"account_id" => buffer.id, "account_code" => "", "debit" => 1800.0, "credit" => 0.0, "description" => ""},
        {"account_id" => gl.id, "account_code" => "", "debit" => 0.0, "credit" => 1800.0, "description" => ""},
      ].to_json
    )
    tx = SaveBankTransaction.create!(
      bank_account_id: bank_account.id,
      date: Time.utc(2026, 3, 9),
      amount: -1800.0,
      currency: "BGN",
      description: "ПЛАЩАНЕ ПО ФАКТУРИ",
      reference: "8598524",
      journal_entry_id: entry.id
    )

    updated = BankTransactionAllocator.reallocate!(company.id, tx, supplier.id, counterpart.id)
    updated.counterpart_id.should eq(counterpart.id)
    updated.document_number.should eq("8598524")
    updated.document_date.should_not be_nil

    lines = JournalLines.parse(updated.lines)
    codes = lines.map { |l| JournalLines.account_code(l) }
    codes.should contain("401")
    codes.should contain("503")
    debit_401 = lines.find { |l| JournalLines.account_code(l) == "401" }
    debit_401.should_not be_nil
    JournalLines.debit(debit_401.not_nil!).should eq(1800.0)
    JournalLines.counterpart_id(debit_401.not_nil!).should eq(counterpart.id)
  end
end
