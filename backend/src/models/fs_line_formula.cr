class FsLineFormula < BaseModel
  table do
    column statement : String
    column line_code : String
    column formula : String = ""

    belongs_to company : Company
  end
end
