class CreateBankAccounts::V20250126000003 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(BankAccount) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to gl_account : Account?, on_delete: :nullify
      add name : String
      add iban : String
      add bic : String?
      add currency : String
      add bank_name : String?
      add integration_type : String, default: "manual"

      # Salt Edge Integration
      add salt_edge_connection_id : String?
      add salt_edge_account_id : String?
      add salt_edge_provider_code : String?
      add salt_edge_last_sync : Time?
      add is_salt_edge_connected : Bool, default: false

      # Wise Integration
      add wise_account_id : String?
      add wise_balance_type : String?
      add wise_last_sync : Time?
      add is_wise_connected : Bool, default: false
    end

    create_index :bank_accounts, [:company_id, :iban], unique: true
  end

  def rollback
    drop table_for(BankAccount)
  end
end
