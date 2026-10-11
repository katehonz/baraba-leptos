class VatExemptionReason < BaseModel
  table do
    column reason_code : String
    column reason_name : String
    column reason_name_en : String?
    column legal_basis : String?
    column legal_basis_en : String?
    column ubl_category_code : String?
    column ubl_exemption_code : String?
    column is_active : Bool = true
    column sort_order : Int32 = 100
  end
end
