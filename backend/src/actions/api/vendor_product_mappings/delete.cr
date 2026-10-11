class Api::VendorProductMappings::Delete < ApiAction
  delete "/api/companies/:company_id/vendor_product_mappings/:mapping_id" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_UPDATE, cid)
    found = VendorProductMappingQuery.new.company_id(cid).id(mapping_id).first?
    if found.nil?
      response.status_code = 404
      return json({success: false, error: "Не е намерено"})
    end
    DeleteVendorProductMapping.delete!(found.not_nil!)
    json({success: true})
  end
end
