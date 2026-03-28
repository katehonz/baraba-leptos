class Api::ScannedInvoices::Update < ApiAction
  put "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)

    SaveScannedInvoice.update(invoice, params) do |operation, updated_invoice|
      if operation.saved?
        json({
          success: true,
          data:    {
            id:                        updated_invoice.id,
            direction:                 updated_invoice.direction,
            status:                    updated_invoice.status,
            vendor_name:               updated_invoice.vendor_name,
            vendor_vat_number:         updated_invoice.vendor_vat_number,
            vendor_address:            updated_invoice.vendor_address,
            customer_name:             updated_invoice.customer_name,
            customer_vat_number:       updated_invoice.customer_vat_number,
            customer_address:          updated_invoice.customer_address,
            invoice_number:            updated_invoice.invoice_number,
            invoice_date:              updated_invoice.invoice_date,
            due_date:                  updated_invoice.due_date,
            subtotal:                  updated_invoice.subtotal,
            total_tax:                 updated_invoice.total_tax,
            invoice_total:             updated_invoice.invoice_total,
            vies_status:               updated_invoice.vies_status,
            vies_company_name:         updated_invoice.vies_company_name,
            counterparty_account_id:   updated_invoice.counterparty_account_id,
            vat_account_id:            updated_invoice.vat_account_id,
            expense_revenue_account_id: updated_invoice.expense_revenue_account_id,
            requires_manual_review:    updated_invoice.requires_manual_review,
            manual_review_reason:      updated_invoice.manual_review_reason,
            notes:                     updated_invoice.notes,
            confidence:                updated_invoice.confidence,
            counterpart_id:            updated_invoice.counterpart_id,
            vat_period:                updated_invoice.vat_period,
            has_inventory:             updated_invoice.has_inventory,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
