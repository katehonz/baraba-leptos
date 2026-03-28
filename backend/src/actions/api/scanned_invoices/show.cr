class Api::ScannedInvoices::Show < ApiAction
  get "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)

    counterpart_name = if cp_id = invoice.counterpart_id
                         CounterpartQuery.new.id(cp_id).first?.try(&.name)
                       end

    json({
      success: true,
      data:    {
        id:                        invoice.id,
        direction:                 invoice.direction,
        status:                    invoice.status,
        vendor_name:               invoice.vendor_name,
        vendor_vat_number:         invoice.vendor_vat_number,
        vendor_address:            invoice.vendor_address,
        customer_name:             invoice.customer_name,
        customer_vat_number:       invoice.customer_vat_number,
        customer_address:          invoice.customer_address,
        invoice_number:            invoice.invoice_number,
        invoice_date:              invoice.invoice_date,
        due_date:                  invoice.due_date,
        subtotal:                  invoice.subtotal,
        total_tax:                 invoice.total_tax,
        invoice_total:             invoice.invoice_total,
        vies_status:               invoice.vies_status,
        vies_validation_message:   invoice.vies_validation_message,
        vies_company_name:         invoice.vies_company_name,
        vies_company_address:      invoice.vies_company_address,
        counterparty_account_id:   invoice.counterparty_account_id,
        vat_account_id:            invoice.vat_account_id,
        expense_revenue_account_id: invoice.expense_revenue_account_id,
        requires_manual_review:    invoice.requires_manual_review,
        manual_review_reason:      invoice.manual_review_reason,
        confidence:                invoice.confidence,
        notes:                     invoice.notes,
        original_file_name:        invoice.original_file_name,
        vat_period:                invoice.vat_period,
        has_inventory:             invoice.has_inventory,
        counterpart_id:            invoice.counterpart_id,
        counterpart_name:          counterpart_name,
        journal_entry_id:          invoice.journal_entry_id,
        created_at:                invoice.created_at,
      },
    })
  end
end
