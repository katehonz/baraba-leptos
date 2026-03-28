class Api::Products::Show < ApiAction
  get "/api/companies/:company_id/products/:product_id" do
    product = ProductQuery.new.company_id(company_id).find(product_id)

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
  end
end
