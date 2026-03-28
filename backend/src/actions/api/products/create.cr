class Api::Products::Create < ApiAction
  post "/api/companies/:company_id/products" do
    SaveProduct.create(params, company_id: company_id.to_i64) do |operation, product|
      if product
        json({
          success: true,
          data:    {
            id:                   product.id,
            code:                 product.code,
            name:                 product.name,
            description:          product.description,
            product_type:         product.product_type,
            price:                product.price,
            measure_unit:         product.measure_unit,
            commodity_code:       product.commodity_code,
            tax_type:             product.tax_type,
            tax_code:             product.tax_code,
            is_active:            product.is_active,
            inventory_account_id: product.inventory_account_id,
            revenue_account_id:   product.revenue_account_id,
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
