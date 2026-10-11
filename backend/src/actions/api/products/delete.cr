class Api::Products::Delete < ApiAction
  delete "/api/companies/:company_id/products/:product_id" do
    product = ProductQuery.new.company_id(company_id).find(product_id)
    DeleteProduct.delete!(product)

    json({
      success: true,
      message: "Product deleted successfully",
    })
  end
end
