# JSONB Serializer Helper for Avram models
#
# This module provides serialization for models with JSON string fields
module JsonbSerializer
  # Parse JSON string safely
  private def self.parse_json(json_str : String) : JSON::Any
    return JSON::Any.new([] of JSON::Any) if json_str.empty?
    begin
      JSON.parse(json_str)
    rescue
      JSON::Any.new([] of JSON::Any)
    end
  end

  # Serialize JournalEntry with lines
  def self.serialize_journal_entry(entry : JournalEntry)
    lines = parse_json(entry.lines)
    {
      id:                     entry.id,
      entry_date:             entry.entry_date.to_s("%Y-%m-%d"),
      description:            entry.description,
      reference:              entry.reference,
      status:                 entry.status,
      document_number:        entry.document_number,
      document_date:          entry.document_date.try(&.to_s("%Y-%m-%d")),
      vat_purchase_operation: entry.vat_purchase_operation,
      vat_sales_operation:    entry.vat_sales_operation,
      total_amount:           entry.total_amount,
      total_vat_amount:       entry.total_vat_amount,
      payment_method_code:    entry.payment_method_code,
      vat_period:             entry.vat_period,
      lines:                  lines,
      company_id:             entry.company_id,
      created_at:             entry.created_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
      updated_at:             entry.updated_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
    }
  end

  # Serialize Invoice with lines
  def self.serialize_invoice(invoice : Invoice)
    lines = parse_json(invoice.lines)
    {
      id:                   invoice.id,
      invoice_number:       invoice.invoice_number,
      document_type:        invoice.document_type,
      original_invoice_id:  invoice.original_invoice_id,
      issue_date:           invoice.issue_date.to_s("%Y-%m-%d"),
      due_date:             invoice.due_date.try(&.to_s("%Y-%m-%d")),
      subtotal:             invoice.subtotal,
      vat_amount:           invoice.vat_amount,
      total_amount:         invoice.total_amount,
      currency:             invoice.currency,
      exchange_rate:        invoice.exchange_rate,
      payment_method:       invoice.payment_method,
      notes:                invoice.notes,
      status:               invoice.status,
      tax_event_date:       invoice.tax_event_date.try(&.to_s("%Y-%m-%d")),
      vat_exemption_reason: invoice.vat_exemption_reason,
      has_inventory:        invoice.has_inventory,
      lines:                lines,
      company_id:           invoice.company_id,
      counterpart_id:       invoice.counterpart_id,
      bank_account_id:      invoice.bank_account_id,
      journal_entry_id:     invoice.journal_entry_id,
      created_at:           invoice.created_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
      updated_at:           invoice.updated_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
    }
  end

  # Serialize Company with settings
  def self.serialize_company(company : Company)
    settings = begin
      JSON.parse(company.settings)
    rescue
      JSON::Any.new({} of String => JSON::Any)
    end

    {
      id:                          company.id,
      name:                        company.name,
      eik:                         company.eik,
      vat_number:                  company.vat_number,
      address:                     company.address,
      city:                        company.city,
      country:                     company.country,
      post_code:                   company.post_code,
      phone:                       company.phone,
      email:                       company.email,
      website:                     company.website,
      manager_name:                company.manager_name,
      manager_eik:                 company.manager_eik,
      manager_egn:                 company.manager_egn,
      accountant_name:             company.accountant_name,
      accountant_egn:              company.accountant_egn,
      authorized_person_name:      company.authorized_person_name,
      authorized_person_egn:       company.authorized_person_egn,
      vat_branch_number:           company.vat_branch_number,
      tax_authority:               company.tax_authority,
      nap_office:                  company.nap_office,
      inventory_valuation_method:  company.inventory_valuation_method,
      is_vat_registered:           company.is_vat_registered,
      vat_period:                  company.vat_period,
      currency:                    company.currency,
      fiscal_year_start_month:     company.fiscal_year_start_month,
      settings:                    settings,
      cash_account_id:             company.cash_account_id,
      bank_account_id:             company.bank_account_id,
      customers_account_id:        company.customers_account_id,
      suppliers_account_id:        company.suppliers_account_id,
      revenues_account_id:         company.revenues_account_id,
      expenses_account_id:         company.expenses_account_id,
      delivery_buffer_account_id:  company.delivery_buffer_account_id,
      vat_receivable_account_id:   company.vat_receivable_account_id,
      vat_payable_account_id:      company.vat_payable_account_id,
      invoice_next_number:         company.invoice_next_number,
      representative_type:         company.representative_type,
      owner_id:                    company.owner_id,
      # SAFt fields
      street_name:                       company.street_name,
      building_number:                   company.building_number,
      region:                            company.region,
      tax_accounting_basis:              company.tax_accounting_basis,
      tax_entity:                        company.tax_entity,
      is_part_of_group:                  company.is_part_of_group,
      beneficial_owner_first_name_bg:    company.beneficial_owner_first_name_bg,
      beneficial_owner_last_name_bg:     company.beneficial_owner_last_name_bg,
      beneficial_owner_egn:              company.beneficial_owner_egn,
      beneficial_owner_first_name_latin: company.beneficial_owner_first_name_latin,
      beneficial_owner_last_name_latin:  company.beneficial_owner_last_name_latin,
      beneficial_owner_country:          company.beneficial_owner_country,
      ultimate_owner_name_bg:            company.ultimate_owner_name_bg,
      ultimate_owner_uic:                company.ultimate_owner_uic,
      ultimate_owner_name_latin:         company.ultimate_owner_name_latin,
      ultimate_owner_country:            company.ultimate_owner_country,
      created_at:                  company.created_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
      updated_at:                  company.updated_at.try(&.to_s("%Y-%m-%d %H:%M:%S")),
    }
  end
end
