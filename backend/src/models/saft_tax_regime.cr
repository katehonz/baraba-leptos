# SAF-T Tax Regime reference table
# Номенклатура на данъчните режими (VAT_TaxType)
class SaftTaxRegime < BaseModel
  table do
    column code : String
    column name : String
    column description : String?
  end
end
