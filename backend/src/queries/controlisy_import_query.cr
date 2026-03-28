class ControlisyImportQuery < ControlisyImport::BaseQuery
  def for_company(company_id : Int64)
    company_id(company_id)
  end

  def recent_first
    created_at.desc_order
  end

  def by_status(status_value : String)
    status(status_value)
  end

  def by_type(type_value : String)
    import_type(type_value)
  end
end
