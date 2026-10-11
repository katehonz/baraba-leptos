class JournalLine < BaseModel
  table do
    column debit_amount : Float64 = 0.0
    column credit_amount : Float64 = 0.0
    column description : String?

    belongs_to journal_entry : JournalEntry
    belongs_to account : Account
  end
end
