class Api::Counterparts::Create < ApiAction
  post "/api/companies/:company_id/counterparts" do
    SaveCounterpart.create(params, company_id: company_id.to_i64) do |operation, counterpart|
      if counterpart
        json({
          success: true,
          data:    {
            id:               counterpart.id,
            name:             counterpart.name,
            vat_number:       counterpart.vat_number,
            address:          counterpart.address,
            contact_person:   counterpart.contact_person,
            email:            counterpart.email,
            phone:            counterpart.phone,
            counterpart_type: counterpart.counterpart_type,
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
