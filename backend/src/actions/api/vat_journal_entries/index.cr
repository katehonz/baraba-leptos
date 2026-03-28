# List VAT journal entries for a VAT return
class Api::VatJournalEntries::Index < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/vat_returns/:vat_return_id/journal_entries" do
    vat_return = VatReturnQuery.new
      .company_id(company_id)
      .id(vat_return_id)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"}, status: 404)
    end

    # Get entry type filter from query params
    entry_type = params.get?(:entry_type)

    query = VatJournalEntryQuery.new
      .vat_return_id(vat_return_id)
      .row_number.asc_order

    if entry_type == "purchase"
      query = query.entry_type("purchase")
    elsif entry_type == "sales"
      query = query.entry_type("sales")
    end

    entries = query.map { |e| journal_entry_json(e) }

    json({
      success: true,
      data:    entries,
    })
  end

  private def journal_entry_json(e : VatJournalEntry)
    {
      id:                     e.id,
      row_number:             e.row_number,
      document_type:          e.document_type,
      document_type_name:     e.document_type_name,
      document_number:        e.document_number,
      document_date:          e.document_date.to_s("%Y-%m-%d"),
      formatted_date:         e.formatted_date,
      counterpart_vat:        e.counterpart_vat,
      counterpart_name:       e.counterpart_name,
      goods_description:      e.goods_description,
      delivery_code:          e.delivery_code,
      delivery_code_name:     e.delivery_code_name,
      entry_type:             e.entry_type,
      # Purchase columns
      base_no_credit:         e.base_no_credit,
      base_full_credit:       e.base_full_credit,
      vat_full_credit:        e.vat_full_credit,
      base_partial_credit:    e.base_partial_credit,
      vat_partial_credit:     e.vat_partial_credit,
      annual_adjustment_base: e.annual_adjustment_base,
      annual_adjustment_vat:  e.annual_adjustment_vat,
      base_20:                e.base_20,
      vat_20:                 e.vat_20,
      vop_base:               e.vop_base,
      vop_vat:                e.vop_vat,
      base_9:                 e.base_9,
      vat_9:                  e.vat_9,
      # Sales columns
      total_base:             e.total_base,
      total_vat:              e.total_vat,
      sales_base_20:          e.sales_base_20,
      sales_vat_20:           e.sales_vat_20,
      sales_base_vop:         e.sales_base_vop,
      sales_vat_vop:          e.sales_vat_vop,
      sales_base_9:           e.sales_base_9,
      sales_vat_9:            e.sales_vat_9,
      sales_base_0_chapter3:  e.sales_base_0_chapter3,
      sales_base_vod:         e.sales_base_vod,
      sales_base_0_articles:  e.sales_base_0_articles,
      sales_base_services_21: e.sales_base_services_21,
      sales_base_69_2:        e.sales_base_69_2,
      sales_base_69_2_eu:     e.sales_base_69_2_eu,
      sales_base_exempt:      e.sales_base_exempt,
      sales_vat_personal:     e.sales_vat_personal,
      sales_base_vop_9:       e.sales_base_vop_9,
      # Metadata
      source_journal_entry_id: e.source_journal_entry_id,
      notes:                   e.notes,
      created_at:              e.created_at,
    }
  end
end
