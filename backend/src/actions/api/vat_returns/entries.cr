# Get VAT journal entries for a VAT return (purchases or sales)
class Api::VatReturns::Entries < ApiAction
  include Api::Auth::SkipRequireAuthToken

  get "/api/companies/:company_id/vat_returns/:vat_return_id/entries" do
    vat_return = VatReturnQuery.new
      .company_id(company_id)
      .id(vat_return_id)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"})
    end

    entry_type = params.get?(:type) || "purchase"

    # Query VatJournalEntry instead of JournalEntry
    entries = VatJournalEntryQuery.new
      .vat_return_id(vat_return.id)
      .entry_type(entry_type)
      .row_number.asc_order

    json({
      success: true,
      data:    entries.map { |e| entry_json(e) },
    })
  end

  private def entry_json(entry : VatJournalEntry)
    # Calculate base and vat amounts based on entry type
    if entry.purchase?
      base_amount = entry.base_full_credit + entry.base_partial_credit + entry.base_no_credit + entry.vop_base
      vat_amount = entry.vat_full_credit + entry.vat_partial_credit + entry.vop_vat
    else
      base_amount = entry.total_base
      vat_amount = entry.total_vat
    end

    {
      id:               entry.id,
      row_number:       entry.row_number,
      document_type:    entry.document_type,
      document_number:  entry.document_number,
      document_date:    entry.document_date.to_s("%Y-%m-%d"),
      counterpart_name: entry.counterpart_name || "",
      counterpart_vat:  entry.counterpart_vat || "",
      goods_description: entry.goods_description || "",
      delivery_code:    entry.delivery_code || "",
      base_amount:      base_amount,
      vat_amount:       vat_amount,
      # Purchase-specific
      base_no_credit:   entry.base_no_credit,
      base_full_credit: entry.base_full_credit,
      vat_full_credit:  entry.vat_full_credit,
      base_partial_credit: entry.base_partial_credit,
      vat_partial_credit: entry.vat_partial_credit,
      annual_adjustment_base: entry.annual_adjustment_base,
      annual_adjustment_vat: entry.annual_adjustment_vat,
      vop_base:         entry.vop_base,
      vop_vat:          entry.vop_vat,
      base_9:           entry.base_9,
      vat_9:            entry.vat_9,
      # Sales-specific
      total_base:       entry.total_base,
      total_vat:        entry.total_vat,
      sales_base_20:    entry.sales_base_20,
      sales_vat_20:     entry.sales_vat_20,
      sales_base_vop:   entry.sales_base_vop,
      sales_vat_vop:    entry.sales_vat_vop,
      sales_base_9:     entry.sales_base_9,
      sales_vat_9:      entry.sales_vat_9,
      sales_base_0_chapter3: entry.sales_base_0_chapter3,
      sales_base_vod:   entry.sales_base_vod,
      sales_base_0_articles: entry.sales_base_0_articles,
      sales_base_services_21: entry.sales_base_services_21,
      sales_base_69_2:  entry.sales_base_69_2,
      sales_base_69_2_eu: entry.sales_base_69_2_eu,
      sales_base_exempt: entry.sales_base_exempt,
      sales_vat_personal: entry.sales_vat_personal,
      sales_base_vop_9: entry.sales_base_vop_9,
    }
  end
end
