class Document < BaseModel
  table do
    column title : String
    column description : String?
    column file_url : String?
    column file_type : String?
    column file_size : Int64?
    column status : String = "pending"  # pending, processed, archived

    belongs_to company : Company
    belongs_to created_by : User?
  end
end
