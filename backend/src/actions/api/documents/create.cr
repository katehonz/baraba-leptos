class Api::Documents::Create < ApiAction
  post "/api/companies/:company_id/documents" do
    SaveDocument.create(params, company_id: company_id.to_i64) do |operation, document|
      if document
        json({
          success: true,
          data:    {
            id:          document.id,
            title:       document.title,
            description: document.description,
            file_url:    document.file_url,
            file_type:   document.file_type,
            status:      document.status,
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
