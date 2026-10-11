class SaveDividend < Dividend::SaveOperation
  permit_columns gross_amount, tax_rate, tax_amount, net_amount, decision_date,
    payment_date, is_paid, notes, company_id, shareholder_id, beneficial_owner_id, dividend_distribution_id

  before_save do
    validate_required gross_amount, tax_rate, decision_date

    # Calculate tax and net amounts if not provided
    if gross_amount.value && tax_rate.value
      gross = gross_amount.value.not_nil!
      rate = tax_rate.value.not_nil!
      calculated_tax = (gross * (rate / 100.0)).round(2)

      if tax_amount.value.nil?
        tax_amount.value = calculated_tax
      end

      if net_amount.value.nil?
        net_amount.value = gross - calculated_tax
      end
    end
  end
end
