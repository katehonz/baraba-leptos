class Api::ScannedInvoices::Index < ApiAction
  get "/api/companies/:company_id/scanned_invoices" do
    invoices = ScannedInvoiceQuery.new.company_id(company_id).created_at.desc_order

    # Filter by status
    if status = params.get?("status")
      invoices = invoices.status(status.to_s)
    end

    # Filter by direction
    if direction = params.get?("direction")
      invoices = invoices.direction(direction.to_s)
    end

    # Filter by VAT period
    if vat_period = params.get?("vat_period")
      invoices = invoices.vat_period(vat_period.to_s)
    end

    json({
      success: true,
      data:    invoices.map { |i|
        counterpart_name = if cp_id = i.counterpart_id
                             CounterpartQuery.new.id(cp_id).first?.try(&.name)
                           end

        {
          id:                     i.id,
          direction:              i.direction,
          status:                 i.status,
          vendor_name:            i.vendor_name,
          vendor_vat_number:      i.vendor_vat_number,
          customer_name:          i.customer_name,
          customer_vat_number:    i.customer_vat_number,
          invoice_number:         i.invoice_number,
          invoice_date:           i.invoice_date,
          subtotal:               i.subtotal,
          total_tax:              i.total_tax,
          invoice_total:          i.invoice_total,
          vies_status:            i.vies_status,
          vies_validation_message: i.vies_validation_message,
          vies_company_name:      i.vies_company_name,
          vies_company_address:   i.vies_company_address,
          confidence:             i.confidence,
          requires_manual_review: i.requires_manual_review,
          manual_review_reason:   i.manual_review_reason,
          counterpart_id:         i.counterpart_id,
          counterpart_name:       counterpart_name,
          journal_entry_id:       i.journal_entry_id,
          vat_period:             i.vat_period,
          payment_method:         i.payment_method,
          payment_method_raw:     i.payment_method_raw,
          payment_journal_entry_id: i.payment_journal_entry_id,
          created_at:             i.created_at,
        }
      },
    })
  end
end
