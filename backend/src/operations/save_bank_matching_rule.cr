class SaveBankMatchingRule < BankMatchingRule::SaveOperation
  permit_columns name, keywords, direction, priority, is_active,
    company_id, account_id, counterpart_id

  before_save do
    validate_required name, keywords, account_id
    if dir = direction.value
      validate_inclusion_of direction, in: ["debit", "credit", "any"] unless dir.blank?
    else
      direction.value = "any"
    end
    if priority.value.nil?
      priority.value = 0
    end
    if is_active.value.nil?
      is_active.value = true
    end
  end
end
