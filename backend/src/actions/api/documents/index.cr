class Api::Documents::Index < ApiAction
  get "/api/companies/:company_id/documents" do
    documents = DocumentQuery.new.company_id(company_id)

    # Filter by status if provided
    if status = params.get?("status")
      documents = documents.status(status.to_s)
    end

    json({
      success: true,
      data:    documents.map { |d| {
        id:          d.id,
        title:       d.title,
        description: d.description,
        file_url:    d.file_url,
        file_type:   d.file_type,
        file_size:   d.file_size,
        status:      d.status,
      } },
    })
  end
end
