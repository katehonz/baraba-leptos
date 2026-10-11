class CreateSaftAssetAccountMappings::V20260130000003 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftAssetAccountMapping) do
      primary_key id : Int64
      add_belongs_to company : Company, on_delete: :cascade
      add asset_transaction_type : String   # SAF-T Asset Transaction Type Code
      add debit_account : String            # Debit account pattern
      add credit_account : String           # Credit account pattern
      add debit_analytical : String?        # Optional debit analytical
      add credit_analytical : String?       # Optional credit analytical
      add description : String?             # Description
      add is_active : Bool, default: true
    end

    # Index for fast lookup by company and transaction type
    execute "CREATE INDEX idx_saft_asset_mappings_company_type ON saft_asset_account_mappings(company_id, asset_transaction_type)"
  end

  def rollback
    drop table_for(SaftAssetAccountMapping)
  end
end
