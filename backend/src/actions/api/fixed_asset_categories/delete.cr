class Api::FixedAssetCategories::Delete < ApiAction
  delete "/api/companies/:company_id/fixed_asset_categories/:fixed_asset_category_id" do
    category = FixedAssetCategoryQuery.new.company_id(company_id).find(fixed_asset_category_id)
    DeleteFixedAssetCategory.delete!(category)
    json({success: true, message: "Category deleted successfully"})
  end
end
