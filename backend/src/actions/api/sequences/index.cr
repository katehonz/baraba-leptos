class Api::Sequences::Index < ApiAction
  get "/api/companies/:company_id/sequences" do
    cid = company_id.to_i64
    authorize!(Permission::SETTINGS_READ, cid)
    company = CompanyQuery.new.find(cid)
    seqs = DocumentNumberService.list_for(company)
    json({
      success: true,
      data:    seqs.map { |seq| DocumentNumberService.serialize(seq) },
      catalog: DocumentNumberService.catalog,
    })
  end
end
