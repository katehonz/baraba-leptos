class AccountingPeriod < BaseModel
  table do
    column year : Int32
    column month : Int32
    column status : String = "open"  # open, closed
    column closed_at : Time?
    column notes : String?

    belongs_to company : Company
    belongs_to closed_by : User?
  end
end
