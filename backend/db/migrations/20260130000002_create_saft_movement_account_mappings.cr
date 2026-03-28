class CreateSaftMovementAccountMappings::V20260130000002 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(SaftMovementAccountMapping) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade

      add movement_type_code : String    # SAF-T movement code (10, 20, 30, etc.)
      add debit_account : String         # Debit account pattern
      add credit_account : String        # Credit account pattern
      add debit_analytical : String?     # Optional analytical debit
      add credit_analytical : String?    # Optional analytical credit
      add description : String?          # Description
      add is_active : Bool, default: true
    end

    # Index for quick lookup by company and movement type
    create_index :saft_movement_account_mappings, [:company_id, :movement_type_code]
    create_index :saft_movement_account_mappings, [:company_id, :debit_account, :credit_account]
  end

  def rollback
    drop table_for(SaftMovementAccountMapping)
  end
end
