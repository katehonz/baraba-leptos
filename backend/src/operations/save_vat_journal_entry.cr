class SaveVatJournalEntry < VatJournalEntry::SaveOperation
  permit_columns(
    row_number,
    document_type,
    document_number,
    document_date,
    counterpart_vat,
    counterpart_name,
    goods_description,
    delivery_code,
    entry_type,
    # Purchase columns
    base_no_credit,
    base_full_credit,
    vat_full_credit,
    base_partial_credit,
    vat_partial_credit,
    annual_adjustment_base,
    annual_adjustment_vat,
    base_20,
    vat_20,
    vop_base,
    vop_vat,
    base_9,
    vat_9,
    # Sales columns
    total_base,
    total_vat,
    sales_base_20,
    sales_vat_20,
    sales_base_vop,
    sales_vat_vop,
    sales_base_9,
    sales_vat_9,
    sales_base_0_chapter3,
    sales_base_vod,
    sales_base_0_articles,
    sales_base_services_21,
    sales_base_69_2,
    sales_base_69_2_eu,
    sales_base_exempt,
    sales_vat_personal,
    sales_base_vop_9,
    # Metadata
    source_journal_entry_id,
    notes,
    company_id,
    vat_return_id
  )

  before_save do
    validate_required row_number, document_type, document_number, document_date, entry_type
    validate_inclusion_of entry_type, in: ["purchase", "sales"]

    # Validate document type based on entry type
    if entry_type.value == "purchase"
      validate_inclusion_of document_type, in: VatJournalEntry::PURCHASE_DOC_TYPES.keys.to_a
    elsif entry_type.value == "sales"
      validate_inclusion_of document_type, in: VatJournalEntry::SALES_DOC_TYPES.keys.to_a
    end

    # Validate delivery code if provided
    if delivery_code.value && !delivery_code.value.to_s.empty?
      validate_inclusion_of delivery_code, in: VatJournalEntry::DELIVERY_CODES.keys.to_a
    end

  end
end
