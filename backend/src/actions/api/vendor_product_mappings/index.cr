class Api::VendorProductMappings::Index < ApiAction
  get "/api/companies/:company_id/vendor_product_mappings" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_READ, cid)

    mappings = VendorProductMappingQuery.new.company_id(cid)
    json({
      success: true,
      data:    mappings.map { |m|
        product = ProductQuery.new.id(m.product_id).first?
        {
          id:                 m.id,
          counterpart_id:     m.counterpart_id,
          vendor_name:        m.vendor_name,
          vendor_sku:         m.vendor_sku,
          vendor_description: m.vendor_description,
          vendor_barcode:     m.vendor_barcode,
          confirmed:          m.confirmed,
          hit_count:          m.hit_count,
          product_id:         m.product_id,
          product_code:       product.try(&.code),
          product_name:       product.try(&.name),
        }
      },
    })
  end
end
