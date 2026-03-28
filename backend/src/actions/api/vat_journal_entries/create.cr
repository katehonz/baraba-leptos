# Create a new VAT journal entry
class Api::VatJournalEntries::Create < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/companies/:company_id/vat_returns/:vat_return_id/journal_entries" do
    cid = company_id.to_i64
    vrid = vat_return_id.to_i64

    # Verify VAT return exists and belongs to company
    vat_return = VatReturnQuery.new
      .company_id(cid)
      .id(vrid)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"}, status: 404)
    end

    # Get next row number
    last_entry = VatJournalEntryQuery.new
      .vat_return_id(vrid)
      .entry_type(params.get(:entry_type))
      .row_number.desc_order
      .first?

    next_row = last_entry ? last_entry.row_number + 1 : 1

    entry = SaveVatJournalEntry.create!(
      company_id: cid,
      vat_return_id: vrid,
      row_number: next_row,
      document_type: params.get(:document_type),
      document_number: params.get(:document_number),
      document_date: Time.parse(params.get(:document_date), "%Y-%m-%d", Time::Location::UTC),
      counterpart_vat: params.get?(:counterpart_vat),
      counterpart_name: params.get?(:counterpart_name),
      goods_description: params.get?(:goods_description),
      delivery_code: params.get?(:delivery_code),
      entry_type: params.get(:entry_type),
      # Purchase columns
      base_no_credit: (params.get?(:base_no_credit) || "0").to_f,
      base_full_credit: (params.get?(:base_full_credit) || "0").to_f,
      vat_full_credit: (params.get?(:vat_full_credit) || "0").to_f,
      base_partial_credit: (params.get?(:base_partial_credit) || "0").to_f,
      vat_partial_credit: (params.get?(:vat_partial_credit) || "0").to_f,
      annual_adjustment_base: (params.get?(:annual_adjustment_base) || "0").to_f,
      annual_adjustment_vat: (params.get?(:annual_adjustment_vat) || "0").to_f,
      base_20: (params.get?(:base_20) || "0").to_f,
      vat_20: (params.get?(:vat_20) || "0").to_f,
      vop_base: (params.get?(:vop_base) || "0").to_f,
      vop_vat: (params.get?(:vop_vat) || "0").to_f,
      base_9: (params.get?(:base_9) || "0").to_f,
      vat_9: (params.get?(:vat_9) || "0").to_f,
      # Sales columns
      total_base: (params.get?(:total_base) || "0").to_f,
      total_vat: (params.get?(:total_vat) || "0").to_f,
      sales_base_20: (params.get?(:sales_base_20) || "0").to_f,
      sales_vat_20: (params.get?(:sales_vat_20) || "0").to_f,
      sales_base_vop: (params.get?(:sales_base_vop) || "0").to_f,
      sales_vat_vop: (params.get?(:sales_vat_vop) || "0").to_f,
      sales_base_9: (params.get?(:sales_base_9) || "0").to_f,
      sales_vat_9: (params.get?(:sales_vat_9) || "0").to_f,
      sales_base_0_chapter3: (params.get?(:sales_base_0_chapter3) || "0").to_f,
      sales_base_vod: (params.get?(:sales_base_vod) || "0").to_f,
      sales_base_0_articles: (params.get?(:sales_base_0_articles) || "0").to_f,
      sales_base_services_21: (params.get?(:sales_base_services_21) || "0").to_f,
      sales_base_69_2: (params.get?(:sales_base_69_2) || "0").to_f,
      sales_base_69_2_eu: (params.get?(:sales_base_69_2_eu) || "0").to_f,
      sales_base_exempt: (params.get?(:sales_base_exempt) || "0").to_f,
      sales_vat_personal: (params.get?(:sales_vat_personal) || "0").to_f,
      sales_base_vop_9: (params.get?(:sales_base_vop_9) || "0").to_f,
      # Metadata
      source_journal_entry_id: params.get?(:source_journal_entry_id).try(&.to_i64),
      notes: params.get?(:notes)
    )

    json({
      success: true,
      data:    {
        id:              entry.id,
        row_number:      entry.row_number,
        document_type:   entry.document_type,
        document_number: entry.document_number,
        entry_type:      entry.entry_type,
      },
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
