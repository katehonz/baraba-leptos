class SaveAccountingPeriod < AccountingPeriod::SaveOperation
  permit_columns year, month, status, closed_at, notes, company_id, closed_by_id

  before_save do
    validate_required year, month, status
    validate_inclusion_of status, in: ["open", "closed"]
  end
end
