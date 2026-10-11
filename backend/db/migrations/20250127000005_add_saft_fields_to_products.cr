class AddSaftFieldsToProducts::V20250127000005 < Avram::Migrator::Migration::V1
  def migrate
    alter table_for(Product) do
      # SAF-T Product fields
      add goods_services_id : String?       # 01=Goods, 02=Services
      add product_group : String?           # Product group
      add product_number_code : String?     # EAN/Barcode
      add valuation_method : String?        # FIFO, LIFO, Average cost, Standard cost
      add uom_base : String?                # Base unit of measure (taxpayer's system)
      add uom_standard : String?            # Standard unit (SAF-T nomenclature)
      add uom_conversion_factor : Float64, default: 1.0
    end
  end

  def rollback
    alter table_for(Product) do
      remove :goods_services_id
      remove :product_group
      remove :product_number_code
      remove :valuation_method
      remove :uom_base
      remove :uom_standard
      remove :uom_conversion_factor
    end
  end
end
