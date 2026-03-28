class FixedAsset < BaseModel
  table do
    column inventory_number : String
    column name : String
    column description : String?
    column acquisition_date : Time
    column put_into_service_date : Time?
    column acquisition_cost : Float64
    column residual_value : Float64?
    column accounting_book_value : Float64?
    column depreciation_method : String  # straight_line, declining_balance
    column accounting_depreciation_rate : Float64
    column tax_depreciation_rate : Float64
    column status : String = "active"  # active, disposed, fully_depreciated
    column document_number : String?
    column document_date : Time?

    # SAP (Счетоводен амортизационен план) - допълнителни полета
    column sap_valuation_class : String?
    column sap_investment_support : Float64 = 0.0
    column sap_asset_life_years : Int32?
    column sap_asset_life_months : Int32?
    column sap_asset_addition : Float64 = 0.0
    column sap_transfers : Float64 = 0.0
    column sap_asset_disposal : Float64 = 0.0
    column sap_book_value_begin : Float64 = 0.0
    column sap_appreciation_for_period : Float64 = 0.0
    column sap_extraordinary_depreciation : Float64 = 0.0
    column sap_accumulated_depreciation : Float64 = 0.0
    column sap_book_value_end : Float64 = 0.0

    # DAP (Данъчен амортизационен план) - всички полета
    column dap_valuation_class : String?
    column dap_category : String?
    column dap_tax_depreciable_value : Float64 = 0.0
    column dap_accrued_tax_depreciation : Float64 = 0.0
    column dap_tax_value_begin : Float64 = 0.0
    column dap_annual_tax_rate : Float64 = 0.0
    column dap_month_change_value : String?
    column dap_month_suspension : String?
    column dap_month_write_off_accounting : String?
    column dap_month_write_off_tax : String?
    column dap_months_depreciation : Int32 = 12
    column dap_depreciation_for_period : Float64 = 0.0
    column dap_accumulated_depreciation : Float64 = 0.0
    column dap_tax_value_end : Float64 = 0.0

    belongs_to company : Company
    belongs_to category : FixedAssetCategory?
    belongs_to supplier : Counterpart?
    belongs_to account : Account?
    has_many asset_transactions : AssetTransaction
  end

  # Calculate depreciation for period (SAP)
  def calculate_sap_depreciation(months : Int32 = 12) : Float64
    return 0.0 if self.sap_book_value_begin <= 0.0
    annual_depreciation = self.sap_book_value_begin * (self.accounting_depreciation_rate / 100.0)
    (annual_depreciation / 12.0) * months
  end

  # Calculate tax depreciation for period (DAP)
  def calculate_dap_depreciation(months : Int32 = 12) : Float64
    return 0.0 if self.dap_tax_value_begin <= 0.0
    annual_depreciation = self.dap_tax_depreciable_value * (self.dap_annual_tax_rate / 100.0)
    (annual_depreciation / 12.0) * months
  end
end
