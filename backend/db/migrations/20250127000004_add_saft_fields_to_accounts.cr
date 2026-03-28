class AddSaftFieldsToAccounts::V20250127000004 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Account) do
      # SAF-T Account fields
      add taxpayer_account_id : String?     # Account code from taxpayer's system
      add grouping_category : String?       # Grouping category (e.g., expense, revenue)
      add grouping_code : String?           # Specific code within category
      add account_creation_date : Time?

      # Opening/Closing Balances (per reporting period)
      add opening_debit_balance : Float64, default: 0.0
      add opening_credit_balance : Float64, default: 0.0
      add closing_debit_balance : Float64, default: 0.0
      add closing_credit_balance : Float64, default: 0.0
    end
  end

  def rollback
    alter table_for(Account) do
      remove :taxpayer_account_id
      remove :grouping_category
      remove :grouping_code
      remove :account_creation_date
      remove :opening_debit_balance
      remove :opening_credit_balance
      remove :closing_debit_balance
      remove :closing_credit_balance
    end
  end
end
