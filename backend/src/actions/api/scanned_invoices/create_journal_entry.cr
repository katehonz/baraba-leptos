class Api::ScannedInvoices::CreateJournalEntry < ApiAction
  post "/api/companies/:company_id/scanned_invoices/:scanned_invoice_id/create_journal_entry" do
    invoice = ScannedInvoiceQuery.new.company_id(company_id).find(scanned_invoice_id)
    company = CompanyQuery.new.find(company_id)

    # Check if already processed
    if invoice.journal_entry_id
      response.status_code = 400
      return json({success: false, error: "Фактурата вече е осчетоводена"})
    end

    # Get account codes
    counterpart_account = if invoice.counterparty_account_id
                            AccountQuery.new.company_id(company_id).id(invoice.counterparty_account_id.not_nil!).first?
                          end

    vat_account = if invoice.vat_account_id
                    AccountQuery.new.company_id(company_id).id(invoice.vat_account_id.not_nil!).first?
                  end

    expense_revenue_account = if invoice.expense_revenue_account_id
                                AccountQuery.new.company_id(company_id).id(invoice.expense_revenue_account_id.not_nil!).first?
                              end

    # Fallback to company defaults if not set
    if counterpart_account.nil?
      account_id = invoice.direction == "purchase" ? company.suppliers_account_id : company.customers_account_id
      counterpart_account = AccountQuery.new.company_id(company_id).id(account_id.not_nil!).first? if account_id
    end

    if expense_revenue_account.nil?
      account_id = invoice.direction == "purchase" ? company.expenses_account_id : company.revenues_account_id
      expense_revenue_account = AccountQuery.new.company_id(company_id).id(account_id.not_nil!).first? if account_id
    end

    if vat_account.nil?
      account_id = invoice.direction == "purchase" ? company.vat_receivable_account_id : company.vat_payable_account_id
      vat_account = AccountQuery.new.company_id(company_id).id(account_id.not_nil!).first? if account_id
    end

    if counterpart_account.nil? || expense_revenue_account.nil?
      response.status_code = 400
      return json({success: false, error: "Не са конфигурирани сметките за осчетоводяване"})
    end

    # Build journal entry lines
    subtotal = invoice.subtotal || 0.0
    vat_amount = invoice.total_tax || 0.0
    total = invoice.invoice_total || (subtotal + vat_amount)

    vendor_or_customer = invoice.direction == "purchase" ? invoice.vendor_name : invoice.customer_name
    invoice_num = invoice.invoice_number || "N/A"

    lines = [] of NamedTuple(account_id: Int64, debit: Float64, credit: Float64, description: String)

    if invoice.direction == "purchase"
      # PURCHASE: Debit Expenses, Debit VAT Receivable, Credit Suppliers
      lines << {
        account_id:  expense_revenue_account.not_nil!.id,
        debit:       subtotal,
        credit:      0.0,
        description: "Разход: #{invoice_num}",
      }

      if vat_amount > 0 && vat_account
        lines << {
          account_id:  vat_account.not_nil!.id,
          debit:       vat_amount,
          credit:      0.0,
          description: "ДДС по фактура #{invoice_num}",
        }
      end

      lines << {
        account_id:  counterpart_account.not_nil!.id,
        debit:       0.0,
        credit:      total,
        description: "Доставчик: #{vendor_or_customer || "N/A"}",
      }
    else
      # SALE: Debit Customers, Credit Revenues, Credit VAT Payable
      lines << {
        account_id:  counterpart_account.not_nil!.id,
        debit:       total,
        credit:      0.0,
        description: "Клиент: #{vendor_or_customer || "N/A"}",
      }

      lines << {
        account_id:  expense_revenue_account.not_nil!.id,
        debit:       0.0,
        credit:      subtotal,
        description: "Приход: #{invoice_num}",
      }

      if vat_amount > 0 && vat_account
        lines << {
          account_id:  vat_account.not_nil!.id,
          debit:       0.0,
          credit:      vat_amount,
          description: "ДДС по фактура #{invoice_num}",
        }
      end
    end

    # Build lines JSON
    lines_json = lines.map do |line|
      {
        "account_id"  => line[:account_id],
        "debit"       => line[:debit],
        "credit"      => line[:credit],
        "description" => line[:description],
      }
    end

    entry_date = invoice.invoice_date || Time.utc
    description = "Фактура #{invoice_num} от #{vendor_or_customer || "N/A"}"

    # Determine VAT period from invoice date
    vat_period = if inv_date = invoice.invoice_date
                   inv_date.to_s("%Y-%m")
                 else
                   Time.utc.to_s("%Y-%m")
                 end

    # Set VAT operation based on direction
    # Default: 01|DOMESTIC_SALE for sales, 01|PURCHASE_CREDIT for purchases
    vat_purchase_op = invoice.direction == "purchase" ? "01|PURCHASE_CREDIT" : nil
    vat_sales_op = invoice.direction == "sale" ? "01|DOMESTIC_SALE" : nil

    # Create journal entry with AI-SCAN reference
    SaveJournalEntry.create(
      company_id: company_id.to_i64,
      entry_date: entry_date,
      description: description,
      reference: "AI-SCAN-#{invoice.id}",
      document_number: invoice.invoice_number,
      document_date: invoice.invoice_date,
      status: "draft",
      lines: lines_json.to_json,
      total_amount: subtotal,
      total_vat_amount: vat_amount,
      counterpart_id: invoice.counterpart_id,
      vat_period: vat_period,
      vat_purchase_operation: vat_purchase_op,
      vat_sales_operation: vat_sales_op
    ) do |operation, entry|
      if entry
        # Update scanned invoice with journal entry ID
        SaveScannedInvoice.update!(invoice,
          journal_entry_id: entry.id,
          status: "processed"
        )

        json({
          success: true,
          data:    {
            journal_entry_id: entry.id,
            message:          "Създаден е счетоводен запис ##{entry.id}",
          },
        })
      else
        response.status_code = 422
        json({success: false, errors: operation.errors.map { |k, e| {field: k, messages: e} }})
      end
    end
  end
end
