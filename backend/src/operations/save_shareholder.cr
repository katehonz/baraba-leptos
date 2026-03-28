class SaveShareholder < Shareholder::SaveOperation
  permit_columns shareholder_type, name, identifier, address, percentage_owned, company_id

  before_save do
    validate_required shareholder_type, name
    validate_inclusion_of shareholder_type, in: ["person", "company"]
  end
end
