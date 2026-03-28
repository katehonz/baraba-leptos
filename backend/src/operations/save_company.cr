class SaveCompany < Company::SaveOperation
  permit_columns name, eik, vat_number, address, city, country, post_code, phone, email, website,
    manager_name, manager_eik, manager_egn,
    accountant_name, accountant_egn,
    authorized_person_name, authorized_person_egn,
    vat_branch_number, representative_type,
    tax_authority, nap_office, is_vat_registered,
    cash_account_id, bank_account_id, customers_account_id, suppliers_account_id,
    vat_payable_account_id, vat_receivable_account_id, revenues_account_id, expenses_account_id,
    owner_id,
    # SAFt fields
    street_name, building_number, region,
    tax_accounting_basis, tax_entity,
    is_part_of_group,
    beneficial_owner_first_name_bg, beneficial_owner_last_name_bg, beneficial_owner_egn,
    beneficial_owner_first_name_latin, beneficial_owner_last_name_latin, beneficial_owner_country,
    ultimate_owner_name_bg, ultimate_owner_uic, ultimate_owner_name_latin, ultimate_owner_country,
    inventory_valuation_method

  before_save do
    validate_required name
    if vat_number.value.presence
      validate_uniqueness_of vat_number
    end
    if eik.value.presence
      validate_uniqueness_of eik
    end
  end
end
