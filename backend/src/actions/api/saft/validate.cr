class Api::Saft::Validate < ApiAction
  include Api::Auth::SkipRequireAuthToken

  param company_id : Int64
  param year : Int32
  param month : Int32

  get "/api/saft/validate" do
    user = current_user?
    return json({error: "Unauthorized"}, status: 401) unless user

    company = CompanyQuery.new.find(company_id)

    # Check authorization (user must own company or have role)
    unless company.owner_id == user.id || user_has_company_role?(user, company)
      return json({error: "Unauthorized"}, status: 403)
    end

    period_start = Time.utc(year, month, 1)
    period_end = period_start.at_end_of_month

    errors = [] of String
    warnings = [] of String

    # Validate company has required fields
    if company.eik.nil? || company.eik.try(&.empty?)
      errors << "Компанията няма попълнен ЕИК"
    end

    if company.is_part_of_group.nil?
      warnings << "Не е попълнена структурата на собственост (OwnershipStructure)"
    end

    # Validate counterparts have SAF-T IDs
    all_counterparts = CounterpartQuery.new.company_id(company.id).is_active.eq(true)
    counterparts_without_saft_id = all_counterparts.select { |c| c.saft_id.nil? || c.saft_id.try(&.empty?) }

    if counterparts_without_saft_id.size > 0
      warnings << "#{counterparts_without_saft_id.size} контрагенти нямат генериран SAF-T ID"
    end

    # Validate accounts have SAF-T mapping
    all_accounts = AccountQuery.new.company_id(company.id).is_active.eq(true)
    accounts_without_saft = all_accounts.select { |a| a.saft_account_id.nil? }

    if accounts_without_saft.size > 0
      warnings << "#{accounts_without_saft.size} сметки нямат SAF-T mapping"
    end

    # Validate journal entries in period
    entries = JournalEntryQuery.new.company_id(company.id).status("posted")
      .entry_date.gte(period_start).entry_date.lte(period_end)

    unbalanced_entries = entries.select { |e| !e.balanced? }
    if unbalanced_entries.size > 0
      errors << "#{unbalanced_entries.size} счетоводни записа не са балансирани"
    end

    entries_without_transaction_id = entries.select { |e| e.transaction_id.nil? || e.transaction_id.try(&.empty?) }
    if entries_without_transaction_id.size > 0
      warnings << "#{entries_without_transaction_id.size} счетоводни записа нямат TransactionID"
    end

    # Validate invoices in period
    invoices = InvoiceQuery.new.company_id(company.id).status("posted")
      .issue_date.gte(period_start).issue_date.lte(period_end)

    invoices_without_transaction_id = invoices.select { |i| i.transaction_id.nil? || i.transaction_id.try(&.empty?) }
    if invoices_without_transaction_id.size > 0
      warnings << "#{invoices_without_transaction_id.size} фактури нямат TransactionID"
    end

    # Summary
    is_valid = errors.empty?

    json({
      valid: is_valid,
      errors: errors,
      warnings: warnings,
      summary: {
        company: company.name,
        period: "#{year}-#{month.to_s.rjust(2, '0')}",
        journal_entries: entries.size,
        invoices: invoices.size,
        counterparts: all_counterparts.size,
        accounts: all_accounts.size
      }
    })
  end

  private def user_has_company_role?(user : User, company : Company) : Bool
    !UserCompanyRoleQuery.new.user_id(user.id).company_id(company.id).first?.nil?
  end
end
