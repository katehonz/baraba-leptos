# Combined Nomenclature - комбинирана номенклатура (NC8/TARIC) за Интрастат
class CombinedNomenclature < BaseModel
  table do
    column cn_code : String
    column description : String
    column primary_unit : String?
    column supplementary_unit : String?
    column level : Int32
    column year : Int32
    column parent_code : String?
    column is_header : Bool = false
  end
end
