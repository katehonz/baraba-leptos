class Api::Shareholders::Create < ApiAction
  post "/api/companies/:company_id/shareholders" do
    SaveShareholder.create(params, company_id: company_id.to_i64) do |operation, shareholder|
      if shareholder
        json({
          success: true,
          data:    {
            id:               shareholder.id,
            shareholder_type: shareholder.shareholder_type,
            name:             shareholder.name,
            identifier:       shareholder.identifier,
            percentage_owned: shareholder.percentage_owned,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
