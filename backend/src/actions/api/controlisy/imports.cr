class Api::Controlisy::Imports < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/controlisy/imports" do
    imports = ControlisyImportQuery.new
      .for_company(company_id.to_i64)
      .recent_first

    json({
      success: true,
      data:    imports.map { |i| import_json(i) },
    })
  end

  private def import_json(import : ControlisyImport)
    {
      id:                    import.id,
      filename:              import.filename,
      import_type:           import.import_type,
      status:                import.status,
      documents_count:       import.documents_count,
      contractors_count:     import.contractors_count,
      journal_entries_count: import.journal_entries_count,
      error_message:         import.error_message,
      imported_at:           import.imported_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
      created_at:            import.created_at.to_s("%Y-%m-%d %H:%M:%S"),
    }
  end
end
