class AssetTransactionQuery < AssetTransaction::BaseQuery
  def by_company(company_id : Int64)
    company_id(company_id)
  end

  def by_asset(fixed_asset_id : Int64)
    fixed_asset_id(fixed_asset_id)
  end

  def by_type(transaction_type : String)
    asset_transaction_type(transaction_type)
  end

  def in_date_range(start_date : Time, end_date : Time)
    transaction_date.gte(start_date).transaction_date.lte(end_date)
  end

  def in_year(year : Int32)
    start_date = Time.utc(year, 1, 1)
    end_date = Time.utc(year, 12, 31, 23, 59, 59)
    in_date_range(start_date, end_date)
  end

  def ordered_by_date
    transaction_date.desc_order
  end
end
