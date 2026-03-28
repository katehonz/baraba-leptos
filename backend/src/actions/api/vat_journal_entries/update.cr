# Update a VAT journal entry
class Api::VatJournalEntries::Update < ApiAction
  include Api::Auth::SkipRequireAuthToken

  put "/api/companies/:company_id/vat_returns/:vat_return_id/journal_entries/:journal_entry_id" do
    cid = company_id.to_i64
    vrid = vat_return_id.to_i64
    eid = journal_entry_id.to_i64

    # Find the entry
    entry = VatJournalEntryQuery.new
      .company_id(cid)
      .vat_return_id(vrid)
      .id(eid)
      .first?

    unless entry
      return json({success: false, error: "Journal entry not found"}, status: 404)
    end

    # Build update params
    update_params = {} of Symbol => String | Int32 | Float64 | Time | Int64 | Nil

    # Update only provided fields
    if doc_type = params.get?(:document_type)
      update_params[:document_type] = doc_type
    end
    if doc_num = params.get?(:document_number)
      update_params[:document_number] = doc_num
    end
    if doc_date = params.get?(:document_date)
      update_params[:document_date] = Time.parse(doc_date, "%Y-%m-%d", Time::Location::UTC)
    end

    # Update the entry
    SaveVatJournalEntry.update!(entry,
      document_type: params.get?(:document_type) || entry.document_type,
      document_number: params.get?(:document_number) || entry.document_number,
      document_date: params.get?(:document_date) ? Time.parse(params.get(:document_date), "%Y-%m-%d", Time::Location::UTC) : entry.document_date,
      counterpart_vat: params.get?(:counterpart_vat) || entry.counterpart_vat,
      counterpart_name: params.get?(:counterpart_name) || entry.counterpart_name,
      goods_description: params.get?(:goods_description) || entry.goods_description,
      delivery_code: params.get?(:delivery_code) || entry.delivery_code,
      # Purchase columns
      base_no_credit: params.get?(:base_no_credit) ? params.get(:base_no_credit).to_f : entry.base_no_credit,
      base_full_credit: params.get?(:base_full_credit) ? params.get(:base_full_credit).to_f : entry.base_full_credit,
      vat_full_credit: params.get?(:vat_full_credit) ? params.get(:vat_full_credit).to_f : entry.vat_full_credit,
      base_partial_credit: params.get?(:base_partial_credit) ? params.get(:base_partial_credit).to_f : entry.base_partial_credit,
      vat_partial_credit: params.get?(:vat_partial_credit) ? params.get(:vat_partial_credit).to_f : entry.vat_partial_credit,
      annual_adjustment_base: params.get?(:annual_adjustment_base) ? params.get(:annual_adjustment_base).to_f : entry.annual_adjustment_base,
      annual_adjustment_vat: params.get?(:annual_adjustment_vat) ? params.get(:annual_adjustment_vat).to_f : entry.annual_adjustment_vat,
      base_20: params.get?(:base_20) ? params.get(:base_20).to_f : entry.base_20,
      vat_20: params.get?(:vat_20) ? params.get(:vat_20).to_f : entry.vat_20,
      vop_base: params.get?(:vop_base) ? params.get(:vop_base).to_f : entry.vop_base,
      vop_vat: params.get?(:vop_vat) ? params.get(:vop_vat).to_f : entry.vop_vat,
      base_9: params.get?(:base_9) ? params.get(:base_9).to_f : entry.base_9,
      vat_9: params.get?(:vat_9) ? params.get(:vat_9).to_f : entry.vat_9,
      # Sales columns
      total_base: params.get?(:total_base) ? params.get(:total_base).to_f : entry.total_base,
      total_vat: params.get?(:total_vat) ? params.get(:total_vat).to_f : entry.total_vat,
      sales_base_20: params.get?(:sales_base_20) ? params.get(:sales_base_20).to_f : entry.sales_base_20,
      sales_vat_20: params.get?(:sales_vat_20) ? params.get(:sales_vat_20).to_f : entry.sales_vat_20,
      sales_base_vop: params.get?(:sales_base_vop) ? params.get(:sales_base_vop).to_f : entry.sales_base_vop,
      sales_vat_vop: params.get?(:sales_vat_vop) ? params.get(:sales_vat_vop).to_f : entry.sales_vat_vop,
      sales_base_9: params.get?(:sales_base_9) ? params.get(:sales_base_9).to_f : entry.sales_base_9,
      sales_vat_9: params.get?(:sales_vat_9) ? params.get(:sales_vat_9).to_f : entry.sales_vat_9,
      sales_base_0_chapter3: params.get?(:sales_base_0_chapter3) ? params.get(:sales_base_0_chapter3).to_f : entry.sales_base_0_chapter3,
      sales_base_vod: params.get?(:sales_base_vod) ? params.get(:sales_base_vod).to_f : entry.sales_base_vod,
      sales_base_0_articles: params.get?(:sales_base_0_articles) ? params.get(:sales_base_0_articles).to_f : entry.sales_base_0_articles,
      sales_base_services_21: params.get?(:sales_base_services_21) ? params.get(:sales_base_services_21).to_f : entry.sales_base_services_21,
      sales_base_69_2: params.get?(:sales_base_69_2) ? params.get(:sales_base_69_2).to_f : entry.sales_base_69_2,
      sales_base_69_2_eu: params.get?(:sales_base_69_2_eu) ? params.get(:sales_base_69_2_eu).to_f : entry.sales_base_69_2_eu,
      sales_base_exempt: params.get?(:sales_base_exempt) ? params.get(:sales_base_exempt).to_f : entry.sales_base_exempt,
      sales_vat_personal: params.get?(:sales_vat_personal) ? params.get(:sales_vat_personal).to_f : entry.sales_vat_personal,
      sales_base_vop_9: params.get?(:sales_base_vop_9) ? params.get(:sales_base_vop_9).to_f : entry.sales_base_vop_9,
      notes: params.get?(:notes) || entry.notes
    )

    json({
      success: true,
      data:    {
        id:              entry.id,
        row_number:      entry.row_number,
        document_type:   entry.document_type,
        document_number: entry.document_number,
      },
    })
  rescue ex
    json({success: false, error: ex.message})
  end
end
