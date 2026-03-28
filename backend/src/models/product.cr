class Product < BaseModel
  table do
    column code : String
    column name : String
    column description : String?
    column product_type : String?      # SAF-T: 10=Materials, 20=Production, 30=Goods, 40=WIP, 50=Investment
    column price : Float64 = 0.0
    column measure_unit : String?      # e.g. "pcs", "kg", "hr"
    column commodity_code : String?    # NC8/TARIC code (8 digits)
    column tax_type : String?
    column tax_code : String?
    column is_active : Bool = true

    # SAF-T Product fields
    column goods_services_id : String?       # 01=Goods, 02=Services
    column product_group : String?           # Product group
    column product_number_code : String?     # EAN/Barcode
    column valuation_method : String?        # FIFO, LIFO, Average cost, Standard cost
    column uom_base : String?                # Base unit of measure (taxpayer's system)
    column uom_standard : String?            # Standard unit (SAF-T nomenclature)
    column uom_conversion_factor : Float64 = 1.0

    belongs_to company : Company
    belongs_to inventory_account : Account?
    belongs_to revenue_account : Account?

    has_many physical_stocks : PhysicalStock
  end

  # Validate commodity code is 8 digits
  def valid_commodity_code? : Bool
    return true if self.commodity_code.nil?
    code = self.commodity_code.not_nil!
    code == "0" || (code.size == 8 && code.chars.all?(&.ascii_number?))
  end
end
