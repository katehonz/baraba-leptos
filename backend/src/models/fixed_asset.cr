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
    column location : String?
    column conserved_from : Time?
    column conserved_to : Time?
    column disposed_at : Time?
    column disposed_reason : String?

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

  def calculate_sap_depreciation(months : Int32 = 12) : Float64
    return 0.0 if accounting_depreciation_rate <= 0.0
    (((acquisition_cost * (accounting_depreciation_rate / 100.0)) / 12.0).round(2) * months)
  end

  def calculate_dap_depreciation(months : Int32 = 12) : Float64
    rate = dap_annual_tax_rate > 0 ? dap_annual_tax_rate : tax_depreciation_rate
    cost = dap_tax_depreciable_value > 0 ? dap_tax_depreciable_value : acquisition_cost
    return 0.0 if rate <= 0.0
    (((cost * (rate / 100.0)) / 12.0).round(2) * months)
  end

  def method_label : String
    depreciation_method == "declining_balance" ? "Намаляващ остатък" : "Линеен"
  end

  def api_json
    {
      id:                           id,
      inventory_number:             inventory_number,
      name:                         name,
      description:                  description,
      category_id:                  category_id,
      acquisition_date:             acquisition_date.to_s("%Y-%m-%d"),
      put_into_service_date:        put_into_service_date.try(&.to_s("%Y-%m-%d")),
      acquisition_cost:             acquisition_cost,
      residual_value:               residual_value,
      accounting_book_value:        accounting_book_value,
      depreciation_method:          depreciation_method,
      accounting_depreciation_rate: accounting_depreciation_rate,
      tax_depreciation_rate:        tax_depreciation_rate,
      status:                       status,
      document_number:              document_number,
      document_date:                document_date.try(&.to_s("%Y-%m-%d")),
      location:                     location,
      conserved_from:               conserved_from.try(&.to_s("%Y-%m-%d")),
      conserved_to:                 conserved_to.try(&.to_s("%Y-%m-%d")),
      disposed_at:                  disposed_at.try(&.to_s("%Y-%m-%d")),
      disposed_reason:              disposed_reason,
      sap_accumulated_depreciation: sap_accumulated_depreciation,
      sap_book_value_begin:         sap_book_value_begin,
      sap_book_value_end:           sap_book_value_end,
      sap_asset_life_months:        sap_asset_life_months,
      dap_category:                 dap_category,
      dap_annual_tax_rate:          dap_annual_tax_rate,
      dap_accrued_tax_depreciation: dap_accrued_tax_depreciation,
      dap_accumulated_depreciation: dap_accumulated_depreciation,
      dap_tax_depreciable_value:    dap_tax_depreciable_value,
      dap_tax_value_begin:          dap_tax_value_begin,
      dap_tax_value_end:            dap_tax_value_end,
      dap_months_depreciation:      dap_months_depreciation,
      dap_month_suspension:         dap_month_suspension,
      dap_month_write_off_accounting: dap_month_write_off_accounting,
      dap_month_write_off_tax:      dap_month_write_off_tax,
    }
  end
end
