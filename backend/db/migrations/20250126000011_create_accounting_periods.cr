class CreateAccountingPeriods::V20250126000011 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(AccountingPeriod) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to closed_by : User?, on_delete: :nullify
      add year : Int32
      add month : Int32
      add status : String, default: "open"
      add closed_at : Time?
      add notes : String?
    end

    create_index :accounting_periods, [:company_id, :year, :month], unique: true
  end

  def rollback
    drop table_for(AccountingPeriod)
  end
end
