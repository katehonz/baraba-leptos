class CreateDividends::V20250126000020 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(Dividend) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to company : Company, on_delete: :cascade
      add_belongs_to shareholder : Shareholder, on_delete: :cascade
      add gross_amount : Float64
      add tax_rate : Float64
      add tax_amount : Float64
      add net_amount : Float64
      add decision_date : Time
      add payment_date : Time?
      add notes : String?
    end

    create_index :dividends, [:company_id, :shareholder_id, :decision_date]
  end

  def rollback
    drop table_for(Dividend)
  end
end
