class Api::Products::Update < ApiAction
  put "/api/companies/:company_id/products/:product_id" do
    product = ProductQuery.new.company_id(company_id).find(product_id)

    SaveProduct.update(product, params) do |operation, updated_product|
      if operation.valid?
        json({
          success: true,
          data:    {
            id:                   updated_product.id,
            code:                 updated_product.code,
            name:                 updated_product.name,
            description:          updated_product.description,
            product_type:         updated_product.product_type,
            price:                updated_product.price,
            measure_unit:         updated_product.measure_unit,
            commodity_code:       updated_product.commodity_code,
            tax_type:             updated_product.tax_type,
            tax_code:             updated_product.tax_code,
            is_active:            updated_product.is_active,
            inventory_account_id: updated_product.inventory_account_id,
            revenue_account_id:   updated_product.revenue_account_id,
          },
        })
      else
        response.status_code = 422
        json({
          success: false,
          errors:  operation.errors.map { |key, errors| {field: key, messages: errors} },
        })
      end
    end
  end
end
