class FixedAssetCategory < BaseModel
  table do
    column name : String
    column description : String?
    column cita_category : String?   # Bulgarian tax category (I, II, III, IV, V, VI, VII)
    column min_depreciation_rate : Float64
    column max_depreciation_rate : Float64
    column default_method : String = "straight_line"  # straight_line, declining_balance

    belongs_to company : Company
    belongs_to asset_account : Account?
    belongs_to depreciation_account : Account?
    belongs_to expense_account : Account?

    has_many fixed_assets : FixedAsset
  end
end
