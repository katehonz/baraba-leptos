class AddSaftFieldsToCounterparts::V20250127000001 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Counterpart) do
      # SAF-T ID (CustomerID/SupplierID format)
      add saft_id_type : String?        # 10, 11, 12, 13, 14, 15, 16
      add saft_id : String?             # Generated SAF-T ID (e.g., 10123456789)

      # SAF-T Address Structure
      add city : String?
      add post_code : String?
      add country_code : String?        # ISO 3166-1 (e.g., BG, DE)
      add region : String?              # ISO 3166-2:BG (e.g., BG-22)
      add street_name : String?
      add building_number : String?
      add additional_address : String?

      # Related Party (свързано лице)
      add is_related_party : Bool, default: false
      add related_party_start_date : Time?
      add related_party_end_date : Time?

      # Opening/Closing Balances (per reporting period)
      add opening_debit_balance : Float64, default: 0.0
      add opening_credit_balance : Float64, default: 0.0
      add closing_debit_balance : Float64, default: 0.0
      add closing_credit_balance : Float64, default: 0.0

      # Self-billing indicator
      add self_billing_indicator : Bool, default: false

      # Default account for this counterpart (for SAF-T AccountID)
      add account_id : Int64?
    end

    create_index :counterparts, [:company_id, :saft_id], unique: true
  end

  def rollback
    alter table_for(Counterpart) do
      remove :saft_id_type
      remove :saft_id
      remove :city
      remove :post_code
      remove :country_code
      remove :region
      remove :street_name
      remove :building_number
      remove :additional_address
      remove :is_related_party
      remove :related_party_start_date
      remove :related_party_end_date
      remove :opening_debit_balance
      remove :opening_credit_balance
      remove :closing_debit_balance
      remove :closing_credit_balance
      remove :self_billing_indicator
      remove :account_id
    end
  end
end
