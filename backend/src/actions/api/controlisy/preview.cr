require "../../../services/controlisy_parser"

class Api::Controlisy::Preview < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/controlisy/preview" do
    # Get the XML content from request body
    xml_content = request.body.try(&.gets_to_end) || ""

    if xml_content.empty?
      response.status_code = 400
      return json({success: false, error: "No XML content provided"})
    end

    # Parse the XML
    begin
      parsed = ControlisyParser.parse(xml_content)
    rescue ex
      response.status_code = 400
      return json({success: false, error: "Invalid XML: #{ex.message}"})
    end

    # Collect all unique account codes from the XML
    account_codes_map = {} of String => String  # code => name
    parsed.documents.each do |doc|
      doc.accountings.each do |acc|
        acc.details.each do |detail|
          unless detail.account_number.empty?
            account_codes_map[detail.account_number] = detail.account_name
          end
        end
      end
    end

    # Get company accounts for auto-mapping
    cid = company_id.to_i64
    company_accounts = AccountQuery.new.company_id(cid)

    # Build account mappings with auto-mapping by code
    account_mappings = account_codes_map.map do |xml_code, xml_name|
      # Try to find matching account by code
      matched_account = company_accounts.find { |acc| acc.code == xml_code }

      if matched_account
        {
          xml_code:            xml_code,
          xml_name:            xml_name,
          mapped_account_id:   matched_account.id.to_i,
          mapped_account_code: matched_account.code,
          mapped_account_name: matched_account.name,
          auto_mapped:         true,
        }
      else
        {
          xml_code:            xml_code,
          xml_name:            xml_name,
          mapped_account_id:   0,
          mapped_account_code: "",
          mapped_account_name: "",
          auto_mapped:         false,
        }
      end
    end

    # Sort mappings by code
    account_mappings = account_mappings.sort_by { |m| m[:xml_code] }

    # Return preview data with account mappings
    json({
      success:          true,
      import_type:      parsed.import_type,
      summary:          {
        contractors_count: parsed.contractors.size,
        documents_count:   parsed.documents.size,
        total_net_amount:  parsed.documents.sum(&.net_amount_bgn.to_f),
        total_vat_amount:  parsed.documents.sum(&.vat_amount_bgn.to_f),
        total_amount:      parsed.documents.sum(&.total_amount_bgn.to_f),
      },
      contractors:      parsed.contractors.first(20).map { |c|
        {
          name:       c.name,
          eik:        c.eik,
          vat_number: c.vat_number,
        }
      },
      documents:        parsed.documents.first(50).map { |d|
        # Collect account codes used in this document
        doc_account_codes = d.accountings.flat_map { |acc|
          acc.details.map(&.account_number)
        }.uniq.reject(&.empty?)

        {
          document_number:   d.document_number,
          document_date:     d.document_date,
          reason:            d.reason || "",
          net_amount:        d.net_amount_bgn.to_f,
          vat_amount:        d.vat_amount_bgn.to_f,
          total_amount:      d.total_amount_bgn.to_f,
          accountings_count: d.accountings.size,
          account_codes:     doc_account_codes,
        }
      },
      account_codes:    account_codes_map.keys.sort,
      account_mappings: account_mappings,
    })
  end
end
