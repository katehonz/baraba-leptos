class SaveAccount < Account::SaveOperation
  permit_columns code, name, account_type, is_active, company_id, saft_account_id, tracks_articles

  before_save do
    validate_required code, name, account_type

    # Normalize account_type to lowercase
    if at = account_type.value
      account_type.value = at.downcase
    end

    # Accept various account type formats
    validate_inclusion_of account_type, in: [
      "asset", "active",
      "liability", "passive",
      "equity",
      "revenue",
      "expense",
      "bifunctional", "active_passive"
    ]
  end
end
