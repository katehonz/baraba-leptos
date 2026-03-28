class Api::Companies::Update < ApiAction
  put "/api/companies/:id" do
    company = CompanyQuery.new.id(id).first?

    if company
      SaveCompany.update(company, params) do |operation, updated_company|
        if updated_company
          json({
            success: true,
            data: JsonbSerializer.serialize_company(updated_company),
          })
        else
          response.status_code = 422
          json({
            success: false,
            errors: operation.errors.map { |key, errors| {field: key, messages: errors} },
          })
        end
      end
    else
      response.status_code = 404
      json({success: false, message: "Company not found"})
    end
  end
end
