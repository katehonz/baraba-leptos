class BankTransaction < BaseModel
  table do
    column date : Time
    column amount : Float64  # Positive = Deposit, Negative = Withdrawal
    column currency : String?
    column amount_base : Float64?  # Amount in base currency
    column exchange_rate : Float64?
    column description : String?
    column contra_account : String?  # IBAN of the other party
    column contra_name : String?     # Name of the other party
    column reference : String?       # Payment reference

    # Integration fields
    column salt_edge_transaction_id : String?
    column salt_edge_category : String?
    column wise_transaction_id : String?

    belongs_to bank_account : BankAccount
    belongs_to journal_entry : JournalEntry?
  end
end
