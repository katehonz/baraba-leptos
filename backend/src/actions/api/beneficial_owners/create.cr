class Api::BeneficialOwners::Create < ApiAction
  post "/api/companies/:company_id/beneficial_owners" do
    SaveBeneficialOwner.create(params, company_id: company_id.to_i64) do |operation, owner|
      if owner
        json({
          success: true,
          data:    {
            id:                   owner.id,
            first_name_bg:        owner.first_name_bg,
            last_name_bg:         owner.last_name_bg,
            egn:                  owner.egn,
            first_name_latin:     owner.first_name_latin,
            last_name_latin:      owner.last_name_latin,
            country:              owner.country,
            ownership_percentage: owner.ownership_percentage,
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
