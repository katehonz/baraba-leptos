# Fixed Asset Transaction - движение на ДМА (амортизация, преоценка, изписване)
class FixedAssetTransaction < BaseModel
  table do
    column transaction_date : Time
    column amount : Float64
    column movement_type : String?      # SAF-T movement type
    column description : String?
    column document_number : String?
    column document_date : Time?

    belongs_to fixed_asset : FixedAsset
    belongs_to company : Company
  end
end
