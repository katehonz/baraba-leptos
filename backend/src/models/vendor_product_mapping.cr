class VendorProductMapping < BaseModel
  table do
    column vendor_name : String?
    column vendor_sku : String?
    column vendor_description : String?
    column vendor_barcode : String?
    column vendor_name_normalized : String?
    column vendor_unit_code : String?
    column conversion_factor : Float64 = 1.0
    column confirmed : Bool = false
    column hit_count : Int32 = 0
    column last_seen_at : Time?

    belongs_to company : Company
    belongs_to counterpart : Counterpart?
    belongs_to product : Product
  end
end
