class Api::Nomenclatures::NapDocumentTypes < ApiAction
  get "/api/nomenclatures/nap_document_types" do
    query = NapDocumentTypeQuery.new
    if journal = params.get?("journal")
      query = query.journal(journal.to_s)
    end
    json({
      success: true,
      data:    query.map { |r|
        {
          code:              r.code,
          journal:           r.journal,
          name_bg:           r.name_bg,
          saft_invoice_type: r.saft_invoice_type,
          is_tax:            r.is_tax,
        }
      },
    })
  end
end
