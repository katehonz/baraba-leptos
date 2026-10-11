class CreateOpeningBalances::V20250126000018 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(OpeningBalance) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to account : Account, on_delete: :cascade
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to accounting_period : AccountingPeriod?, on_delete: :nullify
      add debit : Float64
      add credit : Float64
      add date : Time
    end

    create_index :opening_balances, [:company_id, :account_id, :date], unique: true
  end

  def rollback
    drop table_for(OpeningBalance)
  end
end
