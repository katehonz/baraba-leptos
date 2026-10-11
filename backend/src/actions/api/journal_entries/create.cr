class Api::JournalEntries::Create < ApiAction
  post "/api/companies/:company_id/journal_entries" do
    begin
      body_str = request.body.try(&.gets_to_end) || "{}"
      json_body = JSON.parse(body_str)

      # Accept both nested { journal_entry: {...} } and flat payloads
      data = json_body["journal_entry"]? || json_body

      entry_date_str = data["entry_date"]?.try(&.as_s?) || ""
      if entry_date_str.blank?
        response.status_code = 422
        return json({
          success: false,
          message: "Счетоводната дата е задължителна",
          errors:  [{field: "entry_date", messages: ["is required"]}],
        })
      end

      description = data["description"]?.try(&.as_s?) || ""
      if description.blank?
        response.status_code = 422
        return json({
          success: false,
          message: "Описанието е задължително",
          errors:  [{field: "description", messages: ["is required"]}],
        })
      end

      entry_date = Time.parse(entry_date_str, "%Y-%m-%d", Time::Location::UTC)

      document_date = if dds = data["document_date"]?.try(&.as_s?)
                        dds.blank? ? nil : Time.parse(dds, "%Y-%m-%d", Time::Location::UTC)
                      else
                        nil
                      end

      reference = blank_to_nil(data["reference"]?.try(&.as_s?))
      status = data["status"]?.try(&.as_s?) || "draft"
      document_number = blank_to_nil(data["document_number"]?.try(&.as_s?))
      vat_period = blank_to_nil(data["vat_period"]?.try(&.as_s?))
      vat_purchase_operation = blank_to_nil(data["vat_purchase_operation"]?.try(&.as_s?))
      vat_sales_operation = blank_to_nil(data["vat_sales_operation"]?.try(&.as_s?))
      payment_method_code = blank_to_nil(data["payment_method_code"]?.try(&.as_s?))
      branch_code = blank_to_nil(data["branch_code"]?.try(&.as_s?)) || "0001"

      # lines may arrive as JSON string or as array
      lines = if lines_val = data["lines"]?
                if s = lines_val.as_s?
                  s
                else
                  lines_val.to_json
                end
              else
                "[]"
              end
      lines = JournalLines.enrich(lines, company_id.to_i64)

      counterpart_id = data["counterpart_id"]?.try(&.as_i64?)
      total_amount = data["total_amount"]?.try(&.as_f?)
      total_vat_amount = data["total_vat_amount"]?.try(&.as_f?)
      journal_type = blank_to_nil(data["journal_type"]?.try(&.as_s?))

      entry = SaveJournalEntry.create!(
        company_id: company_id.to_i64,
        entry_date: entry_date,
        document_date: document_date,
        description: description,
        reference: reference,
        status: status,
        document_number: document_number,
        vat_period: vat_period,
        vat_purchase_operation: vat_purchase_operation,
        vat_sales_operation: vat_sales_operation,
        payment_method_code: payment_method_code,
        branch_code: branch_code,
        lines: lines,
        counterpart_id: counterpart_id,
        total_amount: total_amount,
        total_vat_amount: total_vat_amount,
        journal_type: journal_type
      )

      json({
        success: true,
        data:    JsonbSerializer.serialize_journal_entry(entry),
      })
    rescue ex : Avram::InvalidOperationError
      response.status_code = 422
      json({
        success: false,
        message: "Грешка при валидация: #{ex.message}",
        errors:  [{field: "base", messages: [ex.message.to_s]}],
      })
    rescue ex : Time::Format::Error
      response.status_code = 422
      json({
        success: false,
        message: "Невалиден формат на дата (очакван YYYY-MM-DD)",
        errors:  [{field: "entry_date", messages: [ex.message.to_s]}],
      })
    rescue ex
      Log.error { "Journal entry create error: #{ex.message}" }
      response.status_code = 500
      json({
        success: false,
        message: "Грешка при създаване: #{ex.message}",
      })
    end
  end

  private def blank_to_nil(value : String?) : String?
    return nil if value.nil? || value.blank?
    value
  end
end
