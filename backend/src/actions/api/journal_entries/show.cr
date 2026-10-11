class Api::JournalEntries::Show < ApiAction
  get "/api/companies/:company_id/journal_entries/:entry_id" do
    entry = JournalEntryQuery.new.company_id(company_id).id(entry_id).first
    company = CompanyQuery.new.find(company_id)

    db_lines = JournalLineQuery.new.journal_entry_id(entry.id)
    lines_payload = if db_lines.select_count > 0
                      db_lines.map { |l|
                        account_code = AccountQuery.new.id(l.account_id).first?.try(&.code) || ""
                        {
                          id:            l.id,
                          account_id:    l.account_id,
                          account_code:  account_code,
                          debit_amount:  l.debit_amount,
                          credit_amount: l.credit_amount,
                          description:   l.description,
                        }
                      }
                    else
                      id_to_code = {} of Int64 => String
                      AccountQuery.new.company_id(company_id).each do |acc|
                        id_to_code[acc.id] = acc.code
                      end
                      JournalLines.parse(entry.lines).map { |l|
                        {
                          id:            JournalLines.int64(l["id"]?) || 0_i64,
                          account_id:    JournalLines.account_id(l) || 0_i64,
                          account_code:  JournalLines.resolve_account_code(l, id_to_code),
                          debit_amount:  JournalLines.debit(l),
                          credit_amount: JournalLines.credit(l),
                          description:   JournalLines.description(l) || "",
                        }
                      }
                    end

    totals = JournalLines.totals_for(entry)
    payment_id = JournalEntryPayment.existing_payment_id(entry)
    is_payment = JournalEntryPayment.payment_entry?(entry)

    json({
      success: true,
      data:    {
        id:                       entry.id,
        entry_date:               entry.entry_date,
        document_date:            entry.document_date.try(&.to_s("%Y-%m-%d")),
        description:              entry.description,
        reference:                entry.reference,
        status:                   entry.status,
        document_number:          entry.document_number,
        vat_period:               entry.vat_period,
        vat_purchase_operation:   entry.vat_purchase_operation,
        vat_sales_operation:      entry.vat_sales_operation,
        counterpart_id:           entry.counterpart_id,
        payment_method_code:      entry.payment_method_code,
        journal_type:             entry.journal_type,
        lines:                    lines_payload,
        total_debit:              totals[:debit],
        total_credit:             totals[:credit],
        payment_journal_entry_id: payment_id,
        is_payment_entry:         is_payment,
        payment_direction:        is_payment ? nil : JournalEntryPayment.direction_for(entry, company),
        can_create_payment:       JournalEntryPayment.can_create_payment?(entry, company),
      },
    })
  end
end
