class SaveInvoice < Invoice::SaveOperation
  permit_columns invoice_number, issue_date, due_date, subtotal, vat_amount,
    total_amount, currency, status, notes, company_id, counterpart_id,
    document_type, direction, exchange_rate, payment_method, tax_event_date,
    vat_exemption_reason, has_inventory, original_invoice_id, bank_account_id,
    location_id, sequence_id, nap_document_type, uncl1001_code, issued_at,
    cancelled_at, cancellation_reason

  before_save do
    direction.value = "sale" if direction.value.nil? || direction.value.try(&.blank?)
    document_type.value = "INVOICE" if document_type.value.nil? || document_type.value.try(&.blank?)
    blank_invoice_number_to_nil

    validate_required issue_date, subtotal, vat_amount, total_amount, counterpart_id
    validate_required invoice_number unless draft?
    validate_inclusion_of status, in: Invoice::STATUSES
    if sale? && counterpart_id.value
      if cp = CounterpartQuery.new.id(counterpart_id.value.not_nil!).first?
        if cid = company_id.value
          if co = CompanyQuery.new.id(cid).first?
            eik = (cp.eik || "").gsub(/\s+/, "")
            if eik.presence && eik == co.eik.gsub(/\s+/, "")
              counterpart_id.add_error("не може да е същата фирма (ползвайте складов трансфер)")
            end
          end
        end
      end
    end
    validate_inclusion_of direction, in: Invoice::DIRECTIONS
    validate_inclusion_of document_type, in: Invoice::DOCUMENT_TYPES
    validate_nap_document
    validate_currency_code

    validate_sale_number_unique
  end

  private def validate_nap_document
    if document_type.value == "PROFORMA"
      nap_document_type.value = nil
      return
    end
    code = nap_document_type.value
    return if code.nil? || code.not_nil!.blank?
    allowed = if direction.value == "purchase"
                VatJournalEntry::PURCHASE_DOC_TYPES
              else
                VatJournalEntry::SALES_DOC_TYPES
              end
    unless allowed.has_key?(code.not_nil!)
      nap_document_type.add_error("не е валиден вид документ по ППЗДДС за този дневник")
    end
  end

  private def draft? : Bool
    status.value.nil? || status.value == "draft"
  end

  private def sale? : Bool
    (direction.value || "sale") == "sale"
  end

  private def blank_invoice_number_to_nil
    if invoice_number.value.try(&.blank?)
      invoice_number.value = nil
    end
  end

  private def validate_currency_code
    cur = currency.value
    return if cur.nil? || cur.blank?
    allowed = CurrencyQuery.new.is_active(true).map(&.code)
    allowed = ["BGN", "EUR", "USD", "GBP"] if allowed.empty?
    unless allowed.includes?(cur)
      currency.add_error("не е активна валута (виж Валути / курсове)")
    end
  end

  private def validate_sale_number_unique
    number = invoice_number.value
    return if number.nil? || number.blank?
    return unless direction.value == "sale"
    cid = company_id.value
    return unless cid

    validate_uniqueness_of invoice_number, query: InvoiceQuery.new.company_id(cid).direction("sale")
  end
end
