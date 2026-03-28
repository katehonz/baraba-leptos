class Api::UltimateParents::Create < ApiAction
  post "/api/companies/:company_id/ultimate_parents" do
    SaveUltimateParent.create(params, company_id: company_id.to_i64) do |operation, parent|
      if parent
        json({
          success: true,
          data:    {
            id:         parent.id,
            name_bg:    parent.name_bg,
            name_latin: parent.name_latin,
            uic:        parent.uic,
            country:    parent.country,
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
