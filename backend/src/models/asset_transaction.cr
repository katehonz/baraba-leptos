# SAF-T AssetTransaction model for SourceDocuments > AssetTransactions section (annual reporting)
class AssetTransaction < BaseModel
  table do
    column asset_transaction_id : String
    column asset_transaction_type : String  # Code from AssetMovementTypes
    column description : String?
    column transaction_date : Time
    column transaction_id : String?
    column saft_customer_id : String?
    column saft_supplier_id : String?
    column valuation_type : String?
    column acquisition_cost_on_transaction : Float64?
    column book_value_on_transaction : Float64
    column transaction_amount : Float64

    belongs_to company : Company
    belongs_to fixed_asset : FixedAsset
    belongs_to counterpart : Counterpart?
    belongs_to journal_entry : JournalEntry?
  end
end
