class DocumentSequenceBinding < BaseModel
  table do
    column family : String
    column document_type : String

    belongs_to company : Company
    belongs_to document_number_sequence : DocumentNumberSequence
  end
end
