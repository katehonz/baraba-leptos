# Tracks Controlisy XML imports
class ControlisyImport < BaseModel
  table do
    column filename : String
    column import_type : String                    # "purchases", "sales", "bank"
    column status : String = "pending"             # "pending", "processing", "completed", "failed"
    column documents_count : Int32 = 0
    column contractors_count : Int32 = 0
    column journal_entries_count : Int32 = 0
    column error_message : String?
    column imported_at : Time?

    belongs_to company : Company
  end

  def completed?
    status == "completed"
  end

  def failed?
    status == "failed"
  end

  def processing?
    status == "processing"
  end
end
