class SaveDividendDistribution < DividendDistribution::SaveOperation
  permit_columns year, total_amount, decision_date, decision_number, status, notes, company_id

  before_save do
    validate_required year, total_amount, decision_date
    validate_inclusion_of status, in: DividendDistribution::STATUSES.keys
  end
end
