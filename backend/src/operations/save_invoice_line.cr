class SaveInvoiceLine < InvoiceLine::SaveOperation
  permit_columns description, quantity, unit_price, vat_rate, line_total, invoice_id, account_id,
    product_id, unit_code, vat_amount, line_total_with_vat, vat_exemption_reason_id,
    line_number, tax_type, tax_code, discount_percent, discount_amount

  before_save do
    validate_required description, quantity, unit_price
    unit_code.value = unit_code.value.presence || "C62"
    sign = 1.0
    if iid = invoice_id.value
      if inv = InvoiceQuery.new.id(iid).first?
        sign = -1.0 if inv.document_type == "CREDIT_NOTE"
      end
    end
    q = quantity.value || 0.0
    p = unit_price.value || 0.0
    disc = discount_amount.value || 0.0
    net = Money.round(q * p - disc)
    line_total.value = sign * net.abs
    rate = vat_rate.value || 20.0
    vat = Money.round(net.abs * rate / 100.0)
    vat_amount.value = sign * vat
    line_total_with_vat.value = (line_total.value || 0.0) + (vat_amount.value || 0.0)
    ubl = nil.as(String?)
    if vid = vat_exemption_reason_id.value
      ubl = VatExemptionReasonQuery.new.id(vid).first?.try(&.ubl_category_code)
    end
    tax_code.value = Money.saft_tax_code(rate, ubl)
    tax_type.value = tax_type.value.presence || "100"
  end
end
