class PaymentQuery < Payment::BaseQuery
  def by_company(company_id : Int64)
    company_id(company_id)
  end

  def by_period(year : Int32, month : Int32)
    period_year(year).period(month)
  end

  def by_counterpart(counterpart_id : Int64)
    counterpart_id(counterpart_id)
  end

  def by_payment_method(method : String)
    payment_method(method)
  end

  def in_date_range(start_date : Time, end_date : Time)
    transaction_date.gte(start_date).transaction_date.lte(end_date)
  end

  def ordered_by_date
    transaction_date.desc_order
  end
end
