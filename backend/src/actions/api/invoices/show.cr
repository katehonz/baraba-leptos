class Api::Invoices::Show < ApiAction
  include Api::InvoiceParams

  get "/api/companies/:company_id/invoices/:invoice_id" do
    cid = company_id.to_i64
    authorize!(Permission::INVOICE_READ, cid)

    invoice = InvoiceQuery.new.company_id(cid).id(invoice_id).preload_counterpart.first
    lines = InvoiceLineQuery.new.invoice_id(invoice.id)
    company = CompanyQuery.new.find(cid)
    cp = invoice.counterpart
    bank = invoice.bank_account_id.try { |bid| BankAccountQuery.new.id(bid).first? }
    loc = invoice.location_id.try { |lid| CompanyLocationQuery.new.id(lid).first? }
    orig = invoice.original_invoice_id.try { |oid| InvoiceQuery.new.id(oid).first? }

    unit_symbols = {} of String => String
    lines.each do |l|
      next if unit_symbols.has_key?(l.unit_code)
      unit_symbols[l.unit_code] = SaftUnitOfMeasureQuery.new.code(l.unit_code).first?.try(&.symbol) || l.unit_code
    end

    json({
      success: true,
      data:    invoice_header_json(invoice).merge({
        counterpart_name:           cp.try(&.name),
        counterpart_eik:            cp.try(&.eik),
        counterpart_vat:            cp.try(&.vat_number),
        counterpart_address:        cp.try(&.address),
        counterpart_city:           cp.try(&.city),
        counterpart_contact:        cp.try(&.contact_person),
        company_name:               company.name,
        company_eik:                company.eik,
        company_vat:                company.vat_number,
        company_address:            company.address,
        company_city:               company.city,
        company_post_code:          company.post_code,
        company_mol:                company.manager_name,
        location_name:              loc.try(&.name),
        bank_iban:                  bank.try(&.iban),
        bank_name:                  bank.try(&.bank_name),
        bank_bic:                   bank.try(&.bic),
        original_invoice_number:    orig.try(&.invoice_number),
        original_invoice_date:      orig.try(&.issue_date).try(&.to_s("%Y-%m-%d")),
        amount_in_words:            BulgarianNumberToWords.convert(invoice.total_amount, invoice.currency),
        lines:                      lines.map { |l|
          {
            id:                  l.id,
            description:         l.description,
            quantity:            l.quantity,
            unit_price:          l.unit_price,
            unit_code:           l.unit_code,
            unit_symbol:         unit_symbols[l.unit_code]?,
            vat_rate:            l.vat_rate,
            vat_amount:          l.vat_amount,
            line_total:          l.line_total,
            line_total_with_vat: l.line_total_with_vat,
            product_id:               l.product_id,
            account_id:               l.account_id,
            vat_exemption_reason_id:  l.vat_exemption_reason_id,
          }
        },
      }),
    })
  end
end
