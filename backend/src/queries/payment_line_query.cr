class PaymentLineQuery < PaymentLine::BaseQuery
  def by_payment(payment_id : Int64)
    payment_id(payment_id)
  end

  def by_account(account_id : Int64)
    account_id(account_id)
  end

  def by_invoice(invoice_id : Int64)
    invoice_id(invoice_id)
  end

  def debits_only
    debit_credit_indicator("D")
  end

  def credits_only
    debit_credit_indicator("C")
  end
end
