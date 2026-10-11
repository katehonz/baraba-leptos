class VatJournalEntryQuery < VatJournalEntry::BaseQuery
  # Filter by entry type
  def purchases
    entry_type("purchase")
  end

  def sales
    entry_type("sales")
  end

  # Filter by VAT return
  def for_vat_return(vat_return_id : Int64)
    vat_return_id(vat_return_id)
  end

  # Filter entries that are excluded from declaration
  def excluded_from_declaration
    document_type.in(VatJournalEntry::EXCLUDED_FROM_DECLARATION)
  end

  # Filter entries that are included in declaration
  def included_in_declaration
    document_type.not.in(VatJournalEntry::EXCLUDED_FROM_DECLARATION)
  end

  # Order by row number
  def ordered
    row_number.asc_order
  end
end
