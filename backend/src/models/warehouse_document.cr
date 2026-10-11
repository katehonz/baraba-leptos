class WarehouseDocument < BaseModel
  table do
    column document_type : String
    column document_number : String?
    column issue_date : Time
    column status : String = "draft"
    column notes : String?
    column source_id : String?
    column movement_type : String?

    belongs_to company : Company
    belongs_to sequence : DocumentNumberSequence?
    belongs_to from_location : CompanyLocation?
    belongs_to to_location : CompanyLocation?
    belongs_to journal_entry : JournalEntry?
    belongs_to invoice : Invoice?
    belongs_to original_warehouse_document : WarehouseDocument?

    has_many warehouse_document_lines : WarehouseDocumentLine
  end

  def draft?
    status == "draft"
  end
end
