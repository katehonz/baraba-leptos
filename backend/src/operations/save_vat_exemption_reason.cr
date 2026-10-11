class SaveVatExemptionReason < VatExemptionReason::SaveOperation
  permit_columns reason_code, reason_name, reason_name_en, legal_basis, legal_basis_en,
    ubl_category_code, ubl_exemption_code, is_active, sort_order
end
