require "../spec_helper"

def poster_company(name : String, eik : String)
  SaveCompany.create!(name: name, eik: eik, currency: "EUR")
end

def poster_accounts(company : Company)
  customers = SaveAccount.create!(company_id: company.id, code: "411", name: "Клиенти", account_type: "asset")
  suppliers = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
  vat_out = SaveAccount.create!(company_id: company.id, code: "4532", name: "ДДС продажби", account_type: "liability")
  vat_in = SaveAccount.create!(company_id: company.id, code: "4531", name: "ДДС покупки", account_type: "asset")
  old_revenue = SaveAccount.create!(company_id: company.id, code: "709", name: "Стара приходна", account_type: "revenue")
  old_expense = SaveAccount.create!(company_id: company.id, code: "609", name: "Стара разходна", account_type: "expense")
  revenue_a = SaveAccount.create!(company_id: company.id, code: "701", name: "Приходи услуги", account_type: "revenue")
  revenue_b = SaveAccount.create!(company_id: company.id, code: "702", name: "Приходи стоки", account_type: "revenue")
  expense = SaveAccount.create!(company_id: company.id, code: "602", name: "Разходи услуги", account_type: "expense")
  inventory = SaveAccount.create!(company_id: company.id, code: "304", name: "Стоки", account_type: "asset")
  fresh = SaveCompany.update!(company,
    customers_account_id: customers.id,
    suppliers_account_id: suppliers.id,
    vat_payable_account_id: vat_out.id,
    vat_receivable_account_id: vat_in.id,
    revenues_account_id: old_revenue.id,
    expenses_account_id: old_expense.id
  )
  {
    company:     fresh,
    customers:   customers,
    suppliers:   suppliers,
    vat_out:     vat_out,
    vat_in:      vat_in,
    old_revenue: old_revenue,
    old_expense: old_expense,
    revenue_a:   revenue_a,
    revenue_b:   revenue_b,
    expense:     expense,
    inventory:   inventory,
  }
end

def poster_counterpart(company : Company, name : String)
  SaveCounterpart.create!(company_id: company.id, name: name, eik: "131313131", counterpart_type: "both")
end

def poster_product(
  company,
  code,
  name,
  product_type = nil,
  goods_services_id = nil,
  revenue_account_id = nil,
  expense_account_id = nil,
  inventory_account_id = nil
)
  seed_uom
  SaveProduct.create!(
    company_id: company.id,
    code: code,
    name: name,
    product_type: product_type,
    goods_services_id: goods_services_id,
    revenue_account_id: revenue_account_id,
    expense_account_id: expense_account_id,
    inventory_account_id: inventory_account_id,
    uom_standard: "C62",
    measure_unit: "C62"
  )
end

def poster_invoice(company, counterpart, direction, number = nil)
  SaveInvoice.create!(
    company_id: company.id,
    counterpart_id: counterpart.id,
    direction: direction,
    document_type: "INVOICE",
    nap_document_type: "01",
    invoice_number: number,
    issue_date: Time.utc(2026, 10, 1),
    subtotal: 0.0,
    vat_amount: 0.0,
    total_amount: 0.0,
    currency: "EUR",
    status: "draft"
  )
end

def poster_line(invoice, product, price, account_id = nil)
  SaveInvoiceLine.create!(
    invoice_id: invoice.id,
    description: product.name,
    quantity: 1.0,
    unit_price: price,
    vat_rate: 20.0,
    product_id: product.id,
    account_id: account_id
  )
end

def journal_account_ids(entry : JournalEntry) : Array(Int64)
  JournalLines.parse(entry.lines).compact_map { |line| JournalLines.int64(line["account_id"]?) }
end

describe InvoicePoster do
  it "posts a sale to the nomenclature revenue accounts, not the company setting" do
    user = UserFactory.create
    company = poster_company("Номенклатура продажби ООД", "209999011")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Клиент А")
    service = poster_product(company, "SRV", "Услуга", goods_services_id: "02", revenue_account_id: accounts[:revenue_a].id)
    goods = poster_product(company, "GD", "Стоки продажба", product_type: "30", revenue_account_id: accounts[:revenue_b].id)
    invoice = poster_invoice(company, counterpart, "sale")
    poster_line(invoice, service, 100.0)
    poster_line(invoice, goods, 50.0)

    posted = InvoicePoster.post!(invoice, company, user)

    posted.status.should eq("posted")
    entry = JournalEntryQuery.new.find(posted.journal_entry_id.not_nil!)
    ids = journal_account_ids(entry)
    ids.should contain(accounts[:customers].id)
    ids.should contain(accounts[:revenue_a].id)
    ids.should contain(accounts[:revenue_b].id)
    ids.should contain(accounts[:vat_out].id)
    ids.should_not contain(accounts[:old_revenue].id)

    lines = JournalLines.parse(entry.lines)
    credit_a = lines.find { |l| JournalLines.int64(l["account_id"]?) == accounts[:revenue_a].id }.not_nil!
    credit_b = lines.find { |l| JournalLines.int64(l["account_id"]?) == accounts[:revenue_b].id }.not_nil!
    JournalLines.credit(credit_a).should be_close(100.0, 0.001)
    JournalLines.credit(credit_b).should be_close(50.0, 0.001)
  end

  it "posts a purchase to the nomenclature expense account, not the company setting" do
    user = UserFactory.create
    company = poster_company("Номенклатура покупки ООД", "209999012")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Доставчик А")
    service = poster_product(company, "BUY", "Наем", goods_services_id: "02", expense_account_id: accounts[:expense].id)
    invoice = poster_invoice(company, counterpart, "purchase", "SUP-15")
    poster_line(invoice, service, 80.0)

    posted = InvoicePoster.post!(invoice, company, user)
    entry = JournalEntryQuery.new.find(posted.journal_entry_id.not_nil!)
    ids = journal_account_ids(entry)
    ids.should contain(accounts[:suppliers].id)
    ids.should contain(accounts[:expense].id)
    ids.should contain(accounts[:vat_in].id)
    ids.should_not contain(accounts[:old_expense].id)

    debit = JournalLines.parse(entry.lines).find { |l| JournalLines.int64(l["account_id"]?) == accounts[:expense].id }.not_nil!
    JournalLines.debit(debit).should be_close(80.0, 0.001)
  end

  it "debits the inventory account for a stock purchase" do
    user = UserFactory.create
    company = poster_company("Склад покупка ООД", "209999013")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Доставчик Б")
    seed_warehouse_doc_types
    location = SaveCompanyLocation.create!(company_id: company.id, name: "Склад", location_type: "WAREHOUSE", holds_stock: true)
    goods = poster_product(company, "STK", "Болт", product_type: "30", inventory_account_id: accounts[:inventory].id, expense_account_id: accounts[:expense].id)
    invoice = poster_invoice(company, counterpart, "purchase", "SUP-16")
    invoice = SaveInvoice.update!(invoice, location_id: location.id)
    poster_line(invoice, goods, 40.0)

    posted = InvoicePoster.post!(invoice, company, user)
    entry = JournalEntryQuery.new.find(posted.journal_entry_id.not_nil!)
    ids = journal_account_ids(entry)
    ids.should contain(accounts[:inventory].id)
    ids.should_not contain(accounts[:expense].id)
    ids.should_not contain(accounts[:old_expense].id)
  end

  it "uses the line account when it is set on the invoice row" do
    user = UserFactory.create
    company = poster_company("Ред сметка ООД", "209999014")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Клиент Б")
    service = poster_product(company, "OV", "Консултация", goods_services_id: "02", revenue_account_id: accounts[:revenue_a].id)
    invoice = poster_invoice(company, counterpart, "sale")
    poster_line(invoice, service, 10.0, accounts[:revenue_b].id)

    posted = InvoicePoster.post!(invoice, company, user)
    ids = journal_account_ids(JournalEntryQuery.new.find(posted.journal_entry_id.not_nil!))
    ids.should contain(accounts[:revenue_b].id)
    ids.should_not contain(accounts[:revenue_a].id)
    ids.should_not contain(accounts[:old_revenue].id)
  end

  it "refuses to post when the nomenclature has no revenue account" do
    user = UserFactory.create
    company = poster_company("Без сметка ООД", "209999015")
    company = poster_accounts(company)[:company]
    counterpart = poster_counterpart(company, "Клиент В")
    service = poster_product(company, "NO", "Без сметка", goods_services_id: "02")
    invoice = poster_invoice(company, counterpart, "sale")
    poster_line(invoice, service, 10.0)

    expect_raises(InvoicePoster::Error, /приходна сметка/) do
      InvoicePoster.post!(invoice, company, user)
    end
  end

  it "deletes a draft journal entry when the invoice returns to draft" do
    user = UserFactory.create
    company = poster_company("Чернова статия ООД", "209999016")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Клиент Г")
    invoice = poster_invoice(company, counterpart, "purchase", "SUP-17")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      user_id: user.id,
      entry_date: invoice.issue_date,
      description: "Чернова",
      status: "draft",
      lines: [
        {"account_id" => accounts[:expense].id, "debit" => 10.0, "credit" => 0.0},
        {"account_id" => accounts[:suppliers].id, "debit" => 0.0, "credit" => 10.0},
      ].to_json
    )
    invoice = SaveInvoice.update!(invoice, status: "issued", journal_entry_id: entry.id, invoice_number: "SUP-17")

    result = InvoicePoster.return_to_draft!(invoice, company, user)
    result[:invoice].status.should eq("draft")
    result[:invoice].journal_entry_id.should be_nil
    result[:message].should contain("изтрита")
    JournalEntryQuery.new.id(entry.id).first?.should be_nil
  end

  it "reverses a posted entry, keeps the number, and allows a full re-post" do
    user = UserFactory.create
    company = poster_company("Обратно чернова ООД", "209999017")
    accounts = poster_accounts(company)
    company = accounts[:company]
    counterpart = poster_counterpart(company, "Клиент Д")
    service = poster_product(company, "AG", "Абонамент", goods_services_id: "02", revenue_account_id: accounts[:revenue_a].id)
    invoice = poster_invoice(company, counterpart, "sale")
    poster_line(invoice, service, 25.0)

    posted = InvoicePoster.post!(invoice, company, user)
    number = posted.invoice_number.not_nil!
    old_id = posted.journal_entry_id.not_nil!

    result = InvoicePoster.return_to_draft!(posted, company, user)
    result[:invoice].status.should eq("draft")
    result[:invoice].invoice_number.should eq(number)
    result[:invoice].journal_entry_id.should be_nil
    JournalEntryQuery.new.find(old_id).status.should eq("reversed")

    again = InvoicePoster.post!(result[:invoice], company, user)
    again.status.should eq("posted")
    again.invoice_number.should eq(number)
    again.journal_entry_id.should_not eq(old_id)
    journal_account_ids(JournalEntryQuery.new.find(again.journal_entry_id.not_nil!)).should contain(accounts[:revenue_a].id)
  end
end
