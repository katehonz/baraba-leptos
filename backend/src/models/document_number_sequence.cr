class DocumentNumberSequence < BaseModel
  table do
    column code : String
    column name : String
    column current_number : Int64 = 0_i64
    column padding : Int32 = 10
    column is_tax : Bool = false
    column reset_policy : String = "never"

    belongs_to company : Company
    has_many document_sequence_bindings : DocumentSequenceBinding
  end

  # Issued numbers are always 10 digits with leading zeros.
  def formatted(number : Int64) : String
    "%010d" % number
  end
end
