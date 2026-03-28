require "../../../services/mistral_document_service"
require "../../../services/vies_service"

class Api::ScannedInvoices::Scan < ApiAction
  post "/api/companies/:company_id/scanned_invoices/scan" do
    company = CompanyQuery.new.find(company_id)

    # Get parameters
    direction = params.get?("direction").try(&.to_s) || "purchase"
    vat_period = params.get?("vat_period").try(&.to_s)

    # Get file from multipart
    file_data = params.get?("file")
    file_name = params.get?("file_name").try(&.to_s) || "invoice.pdf"

    # Also support base64 encoded file content
    base64_content = params.get?("file_content").try(&.to_s)

    if base64_content.nil? && file_data.nil?
      response.status_code = 400
      return json({success: false, error: "Не е предоставен файл"})
    end

    # Decode file content
    file_bytes = if base64_content
                   Base64.decode(base64_content)
                 else
                   file_data.to_s.to_slice
                 end

    # Get Mistral API key from company settings
    mistral_api_key = company.mistral_api_key.try(&.as_s?) || ENV["MISTRAL_API_KEY"]? || ""

    if mistral_api_key.empty?
      response.status_code = 400
      return json({success: false, error: "Моля конфигурирайте Mistral API ключ в настройките на фирмата"})
    end

    begin
      # Call Mistral Document Service
      result = MistralDocumentService.scan_invoice(
        file_bytes,
        mistral_api_key,
        direction,
        file_name
      )

      invoice_data = result.invoice

      # Get default accounts from company
      counterparty_account_id = direction == "purchase" ? company.suppliers_account_id : company.customers_account_id
      vat_account_id = direction == "purchase" ? company.vat_receivable_account_id : company.vat_payable_account_id
      expense_revenue_account_id = direction == "purchase" ? company.expenses_account_id : company.revenues_account_id

      # Extract VAT number for validation
      vat_number = direction == "purchase" ? invoice_data.vendor_vat_number : invoice_data.customer_vat_number
      counterpart_id : Int64? = nil
      vies_status = "pending"
      vies_validation_message : String? = nil
      vies_company_name : String? = nil
      vies_company_address : String? = nil

      # Validate counterpart: First DB, then VIES
      if vat_number && !vat_number.empty?
        # 1. First search in local database by VAT number
        counterpart = CounterpartQuery.new.company_id(company_id).vat_number(vat_number).first?

        # 2. If not found by VAT, try by EIK (for Bulgarian companies)
        if counterpart.nil? && vat_number.starts_with?("BG")
          eik = vat_number[2..]
          counterpart = CounterpartQuery.new.company_id(company_id).eik(eik).first?
        end

        if counterpart
          # Found in database - no need to validate via VIES
          counterpart_id = counterpart.id
          vies_status = "valid"
          vies_validation_message = "Намерен в базата данни"
          vies_company_name = counterpart.name
          vies_company_address = counterpart.address
        else
          # 3. Not in database - validate via VIES
          begin
            vies_result = ViesService.check_vat(vat_number)
            if vies_result.valid
              vies_status = "valid"
              vies_company_name = vies_result.name
              vies_company_address = vies_result.address
              vies_validation_message = "Валидиран през VIES"

              # 4. Auto-create counterpart from VIES data or invoice data
              # Use VIES name if available, otherwise use name from invoice
              counterpart_name = vies_company_name
              if counterpart_name.nil? || counterpart_name.empty?
                counterpart_name = direction == "purchase" ? invoice_data.vendor_name : invoice_data.customer_name
              end

              counterpart_address = vies_company_address
              if counterpart_address.nil? || counterpart_address.empty?
                counterpart_address = direction == "purchase" ? invoice_data.vendor_address : invoice_data.customer_address
              end

              if counterpart_name && !counterpart_name.empty?
                # Extract EIK from VAT number for Bulgarian companies
                eik = vat_number.starts_with?("BG") ? vat_number[2..] : nil

                # Determine counterpart type based on invoice direction
                cp_type = direction == "purchase" ? "supplier" : "customer"

                new_counterpart = SaveCounterpart.create!(
                  company_id: company_id.to_i64,
                  name: counterpart_name,
                  vat_number: vat_number,
                  eik: eik,
                  address: counterpart_address,
                  counterpart_type: cp_type
                )
                counterpart_id = new_counterpart.id
                vies_company_name = counterpart_name
                vies_company_address = counterpart_address
                vies_validation_message = "Валидиран през VIES и добавен като контрагент"
              end
            else
              vies_status = "invalid"
              vies_validation_message = vies_result.error || "ДДС номерът не е валиден във VIES"
            end
          rescue ex
            vies_status = "error"
            vies_validation_message = "Грешка при VIES валидация: #{ex.message}"
          end
        end
      else
        vies_status = "not_applicable"
        vies_validation_message = "Няма ДДС номер за валидация"
      end

      # Determine if manual review is needed
      requires_manual_review = invoice_data.requires_manual_review
      manual_review_reason = invoice_data.manual_review_reason

      # Additional check: if counterpart not found and VIES invalid, require manual review
      if counterpart_id.nil? && vies_status != "valid"
        requires_manual_review = true
        if manual_review_reason
          manual_review_reason = "#{manual_review_reason}; Контрагентът не е намерен"
        else
          manual_review_reason = "Контрагентът не е намерен в базата данни"
        end
      end

      # Save to database
      SaveScannedInvoice.create(
        company_id: company_id.to_i64,
        direction: invoice_data.direction,
        status: "pending",
        vendor_name: invoice_data.vendor_name,
        vendor_vat_number: invoice_data.vendor_vat_number,
        vendor_address: invoice_data.vendor_address,
        customer_name: invoice_data.customer_name,
        customer_vat_number: invoice_data.customer_vat_number,
        customer_address: invoice_data.customer_address,
        invoice_number: invoice_data.invoice_number,
        invoice_date: invoice_data.invoice_date,
        due_date: invoice_data.due_date,
        subtotal: invoice_data.subtotal,
        total_tax: invoice_data.total_tax,
        invoice_total: invoice_data.invoice_total,
        vies_status: vies_status,
        vies_validation_message: vies_validation_message,
        vies_company_name: vies_company_name,
        vies_company_address: vies_company_address,
        vies_validated_at: vies_status == "valid" ? Time.utc : nil,
        counterparty_account_id: counterparty_account_id,
        vat_account_id: vat_account_id,
        expense_revenue_account_id: expense_revenue_account_id,
        requires_manual_review: requires_manual_review,
        manual_review_reason: manual_review_reason,
        confidence: invoice_data.confidence,
        original_file_name: file_name,
        azure_raw_json: result.raw_json,
        vat_period: vat_period,
        counterpart_id: counterpart_id
      ) do |operation, invoice|
        if invoice
          json({
            success: true,
            data:    {
              id:                         invoice.id,
              direction:                  invoice.direction,
              status:                     invoice.status,
              vendor_name:                invoice.vendor_name,
              vendor_vat_number:          invoice.vendor_vat_number,
              vendor_address:             invoice.vendor_address,
              customer_name:              invoice.customer_name,
              customer_vat_number:        invoice.customer_vat_number,
              customer_address:           invoice.customer_address,
              invoice_number:             invoice.invoice_number,
              invoice_date:               invoice.invoice_date,
              due_date:                   invoice.due_date,
              subtotal:                   invoice.subtotal,
              total_tax:                  invoice.total_tax,
              invoice_total:              invoice.invoice_total,
              confidence:                 invoice.confidence,
              requires_manual_review:     invoice.requires_manual_review,
              manual_review_reason:       invoice.manual_review_reason,
              counterpart_id:             invoice.counterpart_id,
              counterparty_account_id:    invoice.counterparty_account_id,
              vat_account_id:             invoice.vat_account_id,
              expense_revenue_account_id: invoice.expense_revenue_account_id,
              vat_period:                 invoice.vat_period,
              vies_status:                invoice.vies_status,
              vies_validation_message:    invoice.vies_validation_message,
              vies_company_name:          invoice.vies_company_name,
              vies_company_address:       invoice.vies_company_address,
            },
          })
        else
          response.status_code = 422
          json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
        end
      end
    rescue ex
      response.status_code = 500
      json({success: false, error: "Грешка при сканиране: #{ex.message}"})
    end
  end
end
