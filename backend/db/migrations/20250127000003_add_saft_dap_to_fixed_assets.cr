class AddSaftDapToFixedAssets::V20250127000003 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(FixedAsset) do
      # Supplier reference
      add supplier_id : Int64?

      # SAP (Счетоводен амортизационен план) - допълнителни полета
      add sap_valuation_class : String?
      add sap_investment_support : Float64, default: 0.0
      add sap_asset_life_years : Int32?
      add sap_asset_life_months : Int32?
      add sap_asset_addition : Float64, default: 0.0
      add sap_transfers : Float64, default: 0.0
      add sap_asset_disposal : Float64, default: 0.0
      add sap_book_value_begin : Float64, default: 0.0
      add sap_appreciation_for_period : Float64, default: 0.0
      add sap_extraordinary_depreciation : Float64, default: 0.0
      add sap_accumulated_depreciation : Float64, default: 0.0
      add sap_book_value_end : Float64, default: 0.0

      # DAP (Данъчен амортизационен план) - всички полета
      add dap_valuation_class : String?                  # Класификация за данъчни цели
      add dap_category : String?                         # Категория данъчен амортизируем актив
      add dap_tax_depreciable_value : Float64, default: 0.0
      add dap_accrued_tax_depreciation : Float64, default: 0.0
      add dap_tax_value_begin : Float64, default: 0.0
      add dap_annual_tax_rate : Float64, default: 0.0
      add dap_month_change_value : String?               # Месец на промяна в стойността
      add dap_month_suspension : String?                 # Месец на преустановяване
      add dap_month_write_off_accounting : String?       # Месец на отписване (счетоводно)
      add dap_month_write_off_tax : String?              # Месец на отписване (данъчно)
      add dap_months_depreciation : Int32, default: 12
      add dap_depreciation_for_period : Float64, default: 0.0
      add dap_accumulated_depreciation : Float64, default: 0.0
      add dap_tax_value_end : Float64, default: 0.0

      # Asset account mapping
      add account_id : Int64?
    end

    create_index :fixed_assets, [:supplier_id]
    create_index :fixed_assets, [:account_id]
  end

  def rollback
    alter table_for(FixedAsset) do
      remove :supplier_id
      remove :sap_valuation_class
      remove :sap_investment_support
      remove :sap_asset_life_years
      remove :sap_asset_life_months
      remove :sap_asset_addition
      remove :sap_transfers
      remove :sap_asset_disposal
      remove :sap_book_value_begin
      remove :sap_appreciation_for_period
      remove :sap_extraordinary_depreciation
      remove :sap_accumulated_depreciation
      remove :sap_book_value_end
      remove :dap_valuation_class
      remove :dap_category
      remove :dap_tax_depreciable_value
      remove :dap_accrued_tax_depreciation
      remove :dap_tax_value_begin
      remove :dap_annual_tax_rate
      remove :dap_month_change_value
      remove :dap_month_suspension
      remove :dap_month_write_off_accounting
      remove :dap_month_write_off_tax
      remove :dap_months_depreciation
      remove :dap_depreciation_for_period
      remove :dap_accumulated_depreciation
      remove :dap_tax_value_end
      remove :account_id
    end
  end
end
