class Account < BaseModel
  table do
    column code : String
    column name : String
    column account_type : String        # Active, Passive, Bifunctional
    column is_active : Bool = true
    column tracks_articles : Bool = false

    # SAF-T Account fields
    column taxpayer_account_id : String?     # Account code from taxpayer's system
    column grouping_category : String?       # Grouping category (e.g., expense, revenue)
    column grouping_code : String?           # Specific code within category
    column account_creation_date : Time?

    # Opening/Closing Balances (per reporting period)
    column opening_debit_balance : Float64 = 0.0
    column opening_credit_balance : Float64 = 0.0
    column closing_debit_balance : Float64 = 0.0
    column closing_credit_balance : Float64 = 0.0

    belongs_to company : Company
    belongs_to saft_account : SaftAccount?

    has_many counterparts : Counterpart
    has_many payment_lines : PaymentLine
  end

  # Get SAF-T AccountID (uses saft_account code if mapped, otherwise own code)
  def saft_standard_account_id : String
    self.saft_account.try(&.code) || self.code
  end
end
