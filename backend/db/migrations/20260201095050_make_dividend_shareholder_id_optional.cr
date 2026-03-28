class MakeDividendShareholderIdOptional::V20260201095050 < Avram::Migrator::Migration::V1
  def migrate
    make_optional :dividends, :shareholder_id
  end

  def rollback
    make_required :dividends, :shareholder_id
  end
end
