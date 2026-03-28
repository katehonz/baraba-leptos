require "../../../services/controlisy_parser"
require "../../../services/controlisy_importer"

class Api::Controlisy::Import < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/controlisy/import" do
    # Get the XML content from request body
    xml_content = request.body.try(&.gets_to_end) || ""

    if xml_content.empty?
      response.status_code = 400
      return json({success: false, error: "No XML content provided"})
    end

    # Parse first to validate and detect type
    begin
      parsed = ControlisyParser.parse(xml_content)
    rescue ex
      response.status_code = 400
      return json({success: false, error: "Invalid XML: #{ex.message}"})
    end

    cid = company_id.to_i64

    # Create import record
    import_record = SaveControlisyImport.create!(
      company_id: cid,
      filename: "upload_#{Time.utc.to_unix}.xml",
      import_type: parsed.import_type,
      status: "processing"
    )

    # Run the import
    importer = ControlisyImporter.new(cid)
    result = importer.import(xml_content)

    # Update import record
    if result.success && result.errors.empty?
      SaveControlisyImport.update!(import_record,
        status: "completed",
        documents_count: result.documents_imported,
        contractors_count: result.contractors_created + result.contractors_updated,
        journal_entries_count: result.journal_entries_created,
        imported_at: Time.utc
      )
    else
      SaveControlisyImport.update!(import_record,
        status: result.errors.empty? ? "completed" : "failed",
        documents_count: result.documents_imported,
        contractors_count: result.contractors_created + result.contractors_updated,
        journal_entries_count: result.journal_entries_created,
        error_message: result.errors.join("; "),
        imported_at: Time.utc
      )
    end

    json({
      success:                  result.success,
      import_id:                import_record.id,
      import_type:              parsed.import_type,
      contractors_in_file:      parsed.contractors.size,
      documents_in_file:        parsed.documents.size,
      contractors_created:      result.contractors_created,
      contractors_updated:      result.contractors_updated,
      documents_imported:       result.documents_imported,
      journal_entries:          result.journal_entries_created,
      vat_journal_entries:      result.vat_journal_entries_created,
      errors:                   result.errors,
      warnings:                 result.warnings,
    })
  end
end
