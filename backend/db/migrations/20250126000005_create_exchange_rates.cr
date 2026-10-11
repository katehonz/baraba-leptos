class CreateExchangeRates::V20250126000005 < Avram::Migrator::Migration::V1
  def migrate
    create table_for(ExchangeRate) do
      primary_key id : Int64
      add_timestamps
      add_belongs_to from_currency : Currency, on_delete: :cascade
      add_belongs_to to_currency : Currency, on_delete: :cascade
      add_belongs_to created_by : User?, on_delete: :nullify
      add rate : Float64
      add reverse_rate : Float64?
      add valid_date : Time
      add rate_source : String, default: "manual"
      add is_active : Bool, default: true
      add notes : String?
    end

    create_index :exchange_rates, [:from_currency_id, :to_currency_id, :valid_date], unique: true
  end

  def rollback
    drop table_for(ExchangeRate)
  end
end
